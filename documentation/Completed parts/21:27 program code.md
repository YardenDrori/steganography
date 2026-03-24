# פרק 21 — קוד התוכנית על פי סטנדרטים בליווי תיעוד

---

## 21.1 קלט

להרחבה על קלטים ופלטים ברמת השירותים הפנימיים ראה סעיף 15.12. הטבלה הבאה מתארת את הקלטים שהמשתמש מספק לאורך מחזור חיי שימוש מלא במערכת:

| קלט | תיאור |
|-----|--------|
| פרטי הרשמה | שם משתמש, דוא"ל, סיסמה, שם פרטי ושם משפחה |
| פרטי כניסה | דוא"ל או שם משתמש + סיסמה |
| קובץ וידאו (carrier) | קובץ המכיל קודק נתמך (H.264, HEVC, VP9, VP8, AV1, MPEG4) |
| קובץ להטמעה (payload) | כל קובץ בינארי שהמשתמש מעוניין להסתיר |
| הגדרות הטמעה | שיטה (QIM/SS/ISS/STDM), delta, seed, ערוצי YUV, מקדמי DCT לטרגט |
| בקשת חילוץ | מזהה קובץ מוטמע + אותן הגדרות הטמעה שבהן נעשה שימוש בהטמעה המקורית |

## 21.2 פלט

| פלט | תיאור |
|-----|--------|
| Access Token | JWT חתום ב-RS256 — משמש לאימות בכל בקשה לאחר הכניסה |
| Refresh Token | טוקן לחידוש ה-access token — מגיע כ-HttpOnly cookie |
| מטא-דאטה של קובץ | שם, מזהה, `is_carrier`, `is_steg_object` — מוחזר לאחר כל העלאה |
| קובץ וידאו מוטמע | הווידאו המקורי עם המטען החבוי בתוכו, שמור ב-MinIO |
| קובץ מחולץ | המטען המקורי כפי שחולץ מהווידאו המוטמע |

---

## 21.3 שירות האימות — Auth Service

### פונקציות מרכזיות

---

#### 21.3.1 — register_user: דפוס Saga

**קובץ:** `auth_service/src/services/user_service.rs`

```rust
pub async fn register_user(
    pool: &PgPool,
    user_service_url: &str,
    user_name: &str,
    first_name: &str,
    last_name: &str,
    email: &str,
    phone_number: Option<&str>,
    is_male: Option<bool>,
    password: &str,
) -> Result<UserResponse, user_service_error::UserServiceError> {
    // step 1: hash the password
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?.to_string();

    // step 2: create profile in user_service
    let user_profile: UserResponse = client
        .post(format!("{}/users", user_service_url))
        .json(&user_create_request).send().await?
        .json().await?;

    let user_id = user_profile.id;

    // step 3: assign role in auth_service
    match user_repository::add_user_role(pool, user_id, Role::User).await {
        Ok(_) => {}
        Err(e) => {
            compensate_delete_user(&user_service_url, user_id).await?; // rollback
            return Err(UserServiceError::DatabaseError(e));
        }
    }
    Ok(user_profile)
}
```

**דפוס Saga:** ההרשמה כותבת לשני מסדי נתונים שונים. אין transaction מסורתי שיכסה שניהם דרך HTTP. הפתרון הוא saga: אם שלב 3 נכשל, מתבצע rollback ידני על שלב 2.

```rust
pub async fn compensate_delete_user(user_service_url: &str, user_id: i64) -> Result<(), UserServiceError> {
    const ATTEMPTS: u8 = 3;
    let client = reqwest::Client::new();
    for i in 0..ATTEMPTS {
        match client.delete(format!("{}/users/{}", user_service_url, user_id))
            .send().await
        {
            Ok(r) if r.status().is_success() => return Ok(()),
            _ => continue,
        }
    }
    return Err(UserServiceError::ExternalServiceError(format!(
        "{:?}",
        errors
    )));
}
```

שלוש ניסיונות כיוון שהרשת עלולה להיות לא זמינה רגעית. אם כל הניסיונות נכשלים, נשאר נתון "יתום" — פרופיל משתמש ב-user_service ללא רשומת אימות מתאימה. זהו אחד מהמגבלות הידועות של ארכיטקטורת microservices ללא distributed transactions.

**Argon2:** גיבוב הסיסמה משתמש ב-Argon2id — האלגוריתם המומלץ כיום לגיבוב סיסמאות. הוא מעוצב להיות איטי ולצרוך זיכרון רב, מה שהופך brute-force attacks לקשים אפילו עם חומרה ייעודית.

---

#### 21.3.2 — מחזור חיי הטוקן: יצירה ו-Rotation

**קובץ:** `auth_service/src/services/token_service.rs`

**יצירת refresh token:**
```rust
fn generate_random_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..REFRESH_TOKEN_LENGTH)
        .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
        .collect()
}

pub async fn create_refresh_token(
    pool: &PgPool,
    user_id: i64,
    device_info: Option<String>,
    validation_time: i64,
) -> Result<String, UserServiceError> {
    let expires_at = Utc::now() + Duration::minutes(validation_time);
    for attempt in 0..MAX_COLLISION_ATTEMPTS {
        let token = generate_random_token();
        let token_hash = hash_token(&token); // SHA256
        match token_repository::save_refresh_token(
            pool,
            user_id,
            &token_hash,
            expires_at,
            device_info.as_deref().unwrap_or(""),
        )
        .await {
            Ok(_) => return Ok(token), // return plaintext to client
            Err(sqlx::Error::Database(e)) if e.constraint()
                .map_or(false, |c| c.contains("token_hash")) => continue, // hash collision — retry
            Err(e) => return Err(UserServiceError::DatabaseError(e)),
        }
    }
    Err(UserServiceError::DatabaseError(sqlx::Error::RowNotFound))
}
```

מסד הנתונים שומר **hash** של הטוקן, לא את הטוקן עצמו. הלקוח מקבל plaintext. כך אם מסד הנתונים נפרץ, התוקפים מקבלים רק hashes בלתי שמישים. לטוקן של 64 תווים מתוך charset של 62 ערכים יש ~380 ביטי אנטרופיה.

**Token Rotation:**
```rust
pub async fn refresh_access_token(
    pool: &PgPool,
    refresh_token: &str,
    jwt_private_key: &str,
    access_token_validation_time: i64,
    refresh_token_validation_time: i64,
) -> Result<(String, String), UserServiceError> {
    let token_hash = hash_token(refresh_token);
    let stored_token = token_repository::get_refresh_token_by_hash(pool, &token_hash).await?
        .ok_or(UserServiceError::InvalidCredentials)?;

    if stored_token.is_expired() {
        token_repository::revoke_refresh_token(pool, stored_token.id()).await?;
        return Err(UserServiceError::InvalidCredentials);
    }

    // revoke old token before issuing new one
    token_repository::revoke_refresh_token(pool, stored_token.id()).await?;
    let new_access_token = create_access_token(
        user_id,
        &pool,
        jwt_private_key,
        access_token_validation_time,
    )
    .await?;
    let new_refresh_token = create_refresh_token(
        pool,
        user_id,
        stored_token.device_info().map(|s| s.to_string()),
        refresh_token_validation_time,
    )
    .await?;
    Ok((new_access_token, new_refresh_token))
}
```

בכל רענון הטוקן הישן **מבוטל** ומונפק חדש לגמרי. כל refresh token תקין לשימוש יחיד. אם תוקף גנב טוקן ומשתמש בו לפני המשתמש החוקי, השימוש הבא של המשתמש יכשל — המערכת יכולה לזהות שימוש כפול ולהתריע.

---

#### 21.3.3 — JWT: חתימה ואימות RS256

**קבצים:** `auth_service/src/auth/jwt.rs` ו-`shared/global/src/auth/jwt.rs`

**חתימה (auth_service בלבד):**
```rust
pub fn encode_jwt(
    user_id: i64, issued_at: i64, expires_at: i64,
    roles: Roles, private_key_pem: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())?;
    let claims = Claims { sub: user_id, exp: expires_at, iat: issued_at, roles };
    encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)
}
```

**אימות (כל שירות):**
```rust
pub fn verify_jwt(token: &str, public_key_pem: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())?;
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.validate_exp = true;
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}
```

**מדוע RS256?** RS256 הוא חתימה אסימטרית: המפתח הפרטי חותם (רק auth_service מחזיק אותו), המפתח הציבורי מאמת (כל שירות). כל שירות מאמת JWT באופן עצמאי ללא פנייה ל-auth_service בכל בקשה. ה-Claims struct מכיל `sub` (user_id), `exp` (תאריך פקיעה), `iat` (זמן הנפקה), ו-`roles` (`HashSet<Role>`).

---

#### 21.3.4 — JWT Extractors: אבטחה מובנית בטיפוסים

**קובץ:** `shared/global/src/auth/user_extractors.rs`

ב-Axum, `FromRequestParts` הוא trait שמגדיר כיצד לחלץ נתון מבקשת HTTP. יישום שלו על struct מאפשר שימוש ב-struct ישירות בחתימת handler — ה-framework מפעיל את החילוץ לפני הפעלת ה-handler:

```rust
#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync + HasJwtPublicKey,
{
    type Rejection = (StatusCode, Json<ErrorBody>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jwt_public_key = state.jwt_public_key();

        let headers = parts.headers.get("authorization")
            .ok_or((StatusCode::UNAUTHORIZED, Json(ErrorBody::new("Unauthorized"))))?
            .to_str()
            .map_err(|_| (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody::new("Invalid Authorization header format")),
            ))?;

        let token = headers
            .strip_prefix("Bearer ")
            .or_else(|| headers.strip_prefix("bearer "))
            .ok_or((
                StatusCode::BAD_REQUEST,
                Json(ErrorBody::new("Authorization header must use Bearer scheme")),
            ))?;

        let claims = verify_jwt(token, &jwt_public_key)
            .map_err(|_| (StatusCode::UNAUTHORIZED, Json(ErrorBody::new("Invalid token"))))?;

        Ok(AuthenticatedUser(claims.sub))
    }
}
```

**ההשלכה ההנדסית:** כל handler שמצהיר על `AuthenticatedUser` בחתימתו מובטח לקבל `user_id` תקין — אם הטוקן לא תקין, Axum מחזיר 401 אוטומטית לפני שה-handler רץ כלל. אין צורך בבדיקה ידנית בכל handler בנפרד.

בנוסף קיים `AuthenticatedUserWithToken` ששומר גם את הטוקן הגולמי — נחוץ ב-steganography_service שצריך להעביר אותו ל-files_service בשם המשתמש. ו-`RequireAdmin` שמחזיר 403 אם הטוקן תקין אך אין לו תפקיד Admin.

---

## 21.4 שירות הקבצים — Files Service

### פונקציות מרכזיות

---

#### 21.4.1 — detect_is_carrier: גילוי קודק אוטומטי

**קובץ:** `files_service/src/services/files_service.rs`

```rust
const SUPPORTED_CODECS: &[ffmpeg_next::codec::Id] = &[
    ffmpeg_next::codec::Id::H264,  ffmpeg_next::codec::Id::HEVC,
    ffmpeg_next::codec::Id::VP9,   ffmpeg_next::codec::Id::VP8,
    ffmpeg_next::codec::Id::AV1,   ffmpeg_next::codec::Id::MPEG4,
];

async fn detect_is_carrier(bucket: &Bucket, object_key: &str) -> bool {
    let data = match bucket.get_object_range(object_key, 0, Some(2_000_000)).await {
        Ok(d) => d,
        Err(_) => return false,
    };
    let mut tmp = match tempfile::NamedTempFile::new() {
        Ok(f) => f,
        Err(_) => return false,
    };
    if tmp.write_all(data.as_slice()).is_err() { return false; }
    if ffmpeg_next::init().is_err() { return false; }

    let input_ctx = match ffmpeg_next::format::input(tmp.path()) {
        Ok(ctx) => ctx,
        Err(_) => return false,
    };
    let video_stream = match input_ctx.streams().best(ffmpeg_next::media::Type::Video) {
        Some(s) => s,
        None => return false,
    };
    let decoder = match ffmpeg_next::codec::Context::from_parameters(video_stream.parameters())
        .and_then(|ctx| ctx.decoder().video())
    {
        Ok(d) => d,
        Err(_) => return false,
    };

    match decoder.codec() {
        Some(c) => SUPPORTED_CODECS.contains(&c.id()),
        None => false,
    }
}
```

**עיצוב Graceful Degradation:** הפונקציה מחזירה `bool` ולא `Result`. כל שלב כושל מחזיר `false` בלבד. זה מכוון: אם גילוי הקודק נכשל מסיבה טכנית, הקובץ פשוט לא יסומן כ-carrier. החלופה — זריקת שגיאה — הייתה מבטלת את ההעלאה כולה בגלל כשל שאינו קריטי.

**2MB בלבד:** לא צריך להוריד את כל הקובץ. כותרת הקובץ (container header) מכילה מפות זרמים ומידע קודק בבתים הראשונים. 2MB מספיקים לכל פורמט נתמך.

---

## 21.5 שירות הסטגנוגרפיה — Steganography Service

### פונקציות מרכזיות

---

#### 21.5.1 — dct_ii ו-idct_ii: מימוש DCT-II

**קובץ:** `steganography_service/src/services/dct.rs`

פונקציות אלו מממשות את הנוסחה הדו-ממדית שפורטה בסעיף 15.1. מספר החלטות מימוש ראויות לציון מעבר למתמטיקה.

**ייצוג שטוח של מטריצה 4×4:** הפיקסלים מועברים כ-`[u8; 16]` ולא כ-`[[u8; 4]; 4]`. גישה לפיקסל בשורה `i` ועמודה `j` מתבצעת ב-`pixels[i * 4 + j]`. מערך שטוח רציף בזיכרון נקרא ביעילות גבוהה יותר ממצביעים בתוך מצביעים, ומאפשר לקומפיילר לבצע אופטימיזציות נוספות.

**מרכוז ערכי הפיקסלים:** לפני החישוב מופחת 128 מכל פיקסל, כך שהטווח עובר מ-0–255 ל-(-128)–127. זה מאזן את גדלי המקדמים ומבטיח שמקדם ה-DC מייצג סטייה מהממוצע ולא ערך מוחלט.

```rust
pub fn dct_ii(pixels: &[u8; 16]) -> [f64; 16] {
    let mut coefficients = [0.0f64; 16];
    for k_x in 0..4 {
        for k_y in 0..4 {
            let mut sum = 0.0f64;
            for i_x in 0..4 {
                for i_y in 0..4 {
                    let curr_pixel = pixels[i_x * 4 + i_y] as f64 - 128.0;
                    let angle_x = k_x as f64 * (2.0 * i_x as f64 + 1.0) * PI / 8.0;
                    let angle_y = k_y as f64 * (2.0 * i_y as f64 + 1.0) * PI / 8.0;
                    sum += curr_pixel * angle_x.cos() * angle_y.cos();
                }
            }
            coefficients[k_x * 4 + k_y] = sum;
        }
    }
    coefficients
}
```

**תיקון קנה-מידה ב-IDCT:** ניתוח נומרי מראה שמקדם ה-DC (k=0) מתקבל גדול פי 4 ושאר המקדמים גדולים פי 2 מהנדרש. הפתרון: הכפלת כל מקדם בגורם קנה-מידה לפני הסכימה:

```rust
pub fn idct_ii(coefficients: &[f64; 16]) -> [u8; 16] {
    let mut pixels = [0u8; 16];
    for i_x in 0..4 {
        for i_y in 0..4 {
            let mut sum = 0.0f64;
            for k_x in 0..4 {
                for k_y in 0..4 {
                    let curr_coefficient = coefficients[k_x * 4 + k_y];
                    let scale_x = if k_x == 0 { 0.25 } else { 0.5 };
                    let scale_y = if k_y == 0 { 0.25 } else { 0.5 };
                    let angle_x = k_x as f64 * (2.0 * i_x as f64 + 1.0) * PI / 8.0;
                    let angle_y = k_y as f64 * (2.0 * i_y as f64 + 1.0) * PI / 8.0;
                    sum += curr_coefficient * scale_x * angle_x.cos() * scale_y * angle_y.cos();
                }
            }
            pixels[i_x * 4 + i_y] = (sum + 128.0).round().clamp(0.0, 255.0) as u8;
        }
    }
    pixels
}
```

בסיום, ה-`clamp(0.0, 255.0)` הכרחי: שינוי מקדם DCT אחד משפיע על כל 16 הפיקסלים, ועלול לדחוף ערך פיקסל מחוץ לטווח החוקי. ללא חיתוך, cast ל-`u8` היה גורם ל-wrapping — ערך 256 הופך ל-0 — שגיאה ויזואלית בולטת.

---

#### 21.5.2 — QIM: כימות אינדקס מודולציה

**קובץ:** `steganography_service/src/services/qim.rs`

הרעיון המתמטי מוסבר בסעיף 15.1. הנה המימוש:

```rust
pub fn qim_embed_bit(coeff: f64, bit: bool, delta: u8) -> f64 {
    let delta_float = delta as f64;
    if bit {
        return (coeff / delta_float).round() * delta_float;
    } else {
        return ((coeff / delta_float - 0.5).round() + 0.5) * delta_float;
    }
}

pub fn qim_extract_bit(coeff: f64, delta: u8) -> bool {
    let delta_float = delta as f64;
    let dist_true  = (coeff - (coeff / delta_float).round() * delta_float).abs();
    let dist_false = (coeff - ((coeff / delta_float - 0.5).round() + 0.5) * delta_float).abs();
    dist_true < dist_false
}
```

`qim_embed_bit` מעגל את המקדם לרמה הקרובה ביותר של הביט הרצוי. `qim_extract_bit` בודק לאיזו ממערכות הרמות המקדם קרוב יותר — הביט "מקודד" בעמדת המקדם עצמו, ללא צורך במידע נוסף.

---

#### 21.5.3 — generate_unit_vector: יצירת וקטור יחידה סודי

**קובץ:** `steganography_service/src/services/vector.rs`

פונקציה זו היא לב מנגנון הסודיות של שיטות SS/ISS/STDM. ה-seed הוא מחרוזת אך PRNG מקבל מספר — הפתרון הוא גיבוב:

```rust
pub fn generate_unit_vector(seed: String, vec_size: usize) -> Result<Vec<f64>, StegServiceError> {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let hashed_string = hasher.finalize();
    let hashed_seed = u64::from_le_bytes(
        hashed_string[0..8]
            .try_into()
            .map_err(|_| StegServiceError::InvalidPayload)?,
    );

    let mut rng = StdRng::seed_from_u64(hashed_seed);

    let mut return_vec: Vec<f64> = Vec::with_capacity(vec_size);
    let mut squared_sum: f64 = 0.0;
    for _ in 0..vec_size {
        let value = rng.random_range(-1.0f64..1.0f64);
        squared_sum += value * value;
        return_vec.push(value);
    }

    let len_in_space = squared_sum.sqrt();
    if len_in_space == 0.0 {
        return Err(StegServiceError::InvalidPayload);
    }
    for i in 0..vec_size {
        return_vec[i] /= len_in_space;
    }
    Ok(return_vec)
}
```

**מדוע נורמליזציה?** אם הווקטור לא היה מנורמל, המכפלה הפנימית `C·u` הייתה גדולה ביחס ישיר לאורך `u`. delta (נניח 50) היה מתייחס לעוצמת אות שונה בתלות ב-seed, מה שהיה מחייב כיול delta לכל seed בנפרד. כשהאורך תמיד 1, delta תמיד מגדיר בדיוק את עוצמת האות ללא תלות ב-seed.

**אבטחה:** SHA256 לא מוסיף אנטרופיה — 8 הבתים הראשונים בלבד נשמרים כ-u64. הוא רק ממיר מחרוזת ארביטרארית למספר. האבטחה האמיתית נובעת מכך שמי שאינו יודע את ה-seed אינו יכול לשחזר את `u` ולכן אינו יכול לחלץ את הביט.

---

#### 21.5.4 — SS / ISS: פיזור ספקטרום ופיזור ספקטרום משופר

**קובץ:** `steganography_service/src/services/spread_spectrum.rs`

**SS — הטמעה:**
```rust
pub fn spread_spectrum_embed(
    get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
    set_coeff: impl Fn(usize, f64) -> Result<(), StegServiceError>,
    coeff_count: usize,
    seed: String,
    bit_to_embed: bool,
    delta: u8,
) -> Result<(), StegServiceError> {
    let unit_vector = vector::generate_unit_vector(seed, coeff_count)?;
    let bit_to_embed_mult: f64 = if bit_to_embed { 1f64 } else { -1f64 };

    for i in 0..coeff_count {
        let curr_coeff = get_coeff(i)?;
        let embedded_coeff = curr_coeff + (bit_to_embed_mult * delta as f64) * unit_vector[i];
        set_coeff(i, embedded_coeff)?;
    }
    Ok(())
}
```

כל מקדם מקבל תוספת `±δ·u[i]`. השינוי בכל מקדם בודד קטן, אך כולם "נדחפים" לאותו כיוון `u`.

**ISS — יעד מדויק:**

ב-SS אין שליטה על הערך הסופי של המכפלה הפנימית — רק על הכמה היא זזה. ISS שולטת על היעד:

```rust
pub fn improved_spread_spectrum_embed(
    get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
    set_coeff: impl Fn(usize, f64) -> Result<(), StegServiceError>,
    coeff_count: usize,
    seed: String,
    bit_to_embed: bool,
    delta: u8,
) -> Result<(), StegServiceError> {
    let unit_vector = vector::generate_unit_vector(seed, coeff_count)?;
    let bit_to_embed_mult: f64 = if bit_to_embed { 1f64 } else { -1f64 };
    let dot_product = calculate_dot_product(&get_coeff, coeff_count, &unit_vector)?;

    for i in 0..coeff_count {
        let curr_coeff = get_coeff(i)?;
        let embedded_coeff =
            curr_coeff + ((bit_to_embed_mult * delta as f64) - dot_product) * unit_vector[i];
        set_coeff(i, embedded_coeff)?;
    }
    Ok(())
}
```

ההבדל: `(±δ - d) · u[i]` במקום `±δ · u[i]`, כאשר `d` היא המכפלה הפנימית הנוכחית. כך המכפלה הפנימית תמיד מגיעה בדיוק ל-±δ ללא קשר לתוכן הבלוק (הוכחה מתמטית בסעיף 15.1).

**חילוץ — משותף ל-SS וISS:**
```rust
pub fn spread_spectrum_extract(
    get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
    coeff_count: usize,
    seed: String,
) -> Result<bool, StegServiceError> {
    let unit_vector = vector::generate_unit_vector(seed, coeff_count)?;
    let dot_product = calculate_dot_product(&get_coeff, coeff_count, &unit_vector)?;
    Ok(dot_product > 0f64)
}
```

פונקציית החילוץ זהה לשתי השיטות כיוון ששתיהן מבוססות על אותו עיקרון: סימן המכפלה הפנימית קובע את הביט. ב-`EmbedMethods::extract`, גם `SS` וגם `ISS` מפנים לאותה פונקציה.

---

#### 21.5.5 — STDM: שילוב SS ו-QIM

**קובץ:** `steganography_service/src/services/stdm.rs`

```rust
pub fn stdm_embed(
    get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
    set_coeff: impl Fn(usize, f64) -> Result<(), StegServiceError>,
    coeff_count: usize,
    seed: String,
    bit_to_embed: bool,
    delta: u8,
) -> Result<(), StegServiceError> {
    let unit_vector = vector::generate_unit_vector(seed, coeff_count)?;
    let original_dot_product =
        vector::calculate_dot_product(&get_coeff, coeff_count, &unit_vector)?;
    let embedded_dot_product = qim::qim_embed_bit(original_dot_product, bit_to_embed, delta);
    vector::do_back_projection_on_coeffs(
        get_coeff, set_coeff, coeff_count, &unit_vector,
        original_dot_product, embedded_dot_product,
    )?;
    Ok(())
}
```

שלושה שלבים: חישוב המכפלה הפנימית הנוכחית, הפעלת QIM על המכפלה הפנימית (לא על מקדם ישיר), ו-back-projection שמפזר את ההפרש חזרה על המקדמים:

```rust
pub fn do_back_projection_on_coeffs(
    get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
    set_coeff: impl Fn(usize, f64) -> Result<(), StegServiceError>,
    coeff_count: usize,
    unit_vector: &[f64],
    original_dot_operation_value: f64,
    modified_dot_operation_value: f64,
) -> Result<(), StegServiceError> {
    let dot_diff = modified_dot_operation_value - original_dot_operation_value;
    for i in 0..coeff_count {
        set_coeff(i, get_coeff(i)? + dot_diff * unit_vector[i])?;
    }
    Ok(())
}
```

כיוון השינוי מוגבל לכיוון `u` בלבד — זה ממזער את הנזק הויזואלי. ה-back-projection הוא ה"קסם" שמאחד SS (פיזור) עם QIM (דיוק בבחירת הרמה).

---

#### 21.5.6 — EmbedMethods: Enum כ-Strategy Pattern

**קובץ:** `steganography_service/src/dtos.rs`

ב-Rust, enum עם `impl` מאפשר פולימורפיזם ללא ירושה. ה-`EmbedMethods` enum מייצג את ארבע שיטות ההטמעה ומספק dispatch גנרי:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EmbedMethods { QIM, STDM, SS, ISS }

impl EmbedMethods {
    pub fn embed(
        &self,
        get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
        set_coeff: impl Fn(usize, f64) -> Result<(), StegServiceError>,
        coeff_count: usize,
        seed: Option<String>,
        bit: bool,
        delta: u8,
    ) -> Result<(), StegServiceError> {
        match self {
            EmbedMethods::QIM  => { qim_embed(get_coeff, set_coeff, bit, delta)?; Ok(()) }
            EmbedMethods::SS   => { spread_spectrum_embed(get_coeff, set_coeff, coeff_count,
                                       seed.ok_or(StegServiceError::InvalidPayload)?, bit, delta)?; Ok(()) }
            EmbedMethods::ISS => {
                improved_spread_spectrum_embed(
                    get_coeff,
                    set_coeff,
                    coeff_count,
                    seed.ok_or(StegServiceError::InvalidPayload)?,
                    bit,
                    delta,
                )?;
                Ok(())
            }
            EmbedMethods::STDM => {
                stdm_embed(
                    get_coeff,
                    set_coeff,
                    coeff_count,
                    seed.ok_or(StegServiceError::InvalidPayload)?,
                    bit,
                    delta,
                )?;
                Ok(())
            }
        }
    }

    pub fn extract(
        &self,
        get_coeff: impl Fn(usize) -> Result<f64, StegServiceError>,
        coeff_count: usize,
        seed: Option<String>,
        delta: u8,
    ) -> Result<bool, StegServiceError> {
        match self {
            Self::QIM => {
                let coeff = get_coeff(0)?;
                let found_bit = qim_extract_bit(coeff, delta);
                return Ok(found_bit);
            }
            Self::STDM => {
                let found_bit = stdm_extract(
                    get_coeff,
                    coeff_count,
                    seed.ok_or(StegServiceError::InvalidPayload)?,
                    delta,
                )?;
                return Ok(found_bit);
            }
            Self::SS | Self::ISS => {
                let found_bit = spread_spectrum_extract(
                    get_coeff,
                    coeff_count,
                    seed.ok_or(StegServiceError::InvalidPayload)?,
                )?;
                return Ok(found_bit);
            }
        };
    }
}
```

הקוד שקורא ל-`method.embed(...)` לא יודע ולא צריך לדעת איזו שיטה מופעלת — זהו דפוס ה-Strategy Pattern ממומש בכלים של Rust. `match` על enum במקום ירושת מחלקה, `impl` על enum במקום abstract class.

---

#### 21.5.7 — process_frame: פונקציה גנרית מסדר גבוה

**קובץ:** `steganography_service/src/services/process_frame.rs`

```rust
pub fn process_frame<F, T, W>(
    configs: &EmbedConfigs,
    state: &mut W,
    buffer: &mut T,
    frame: &mut ffmpeg_next::frame::Video,
    channel_method: F,
) -> Result<(), StegServiceError>
where
    F: Fn(&EmbedConfigs, &mut W, &mut T, &mut ffmpeg_next::frame::Video,
          u32, u32, usize) -> Result<(), StegServiceError>,
{
    if let Some(yuv) = &configs.channels_to_embed.yuv {
        let (y_w, y_h, u_w, u_h, v_w, v_h) = find_dimensions_for_codec(frame, configs)?;
        if yuv.y  { channel_method(configs, state, buffer, frame, y_w, y_h, Y_PLANE)?;  }
        if yuv.cb { channel_method(configs, state, buffer, frame, u_w, u_h, CB_PLANE)?; }
        if yuv.cr { channel_method(configs, state, buffer, frame, v_w, v_h, CR_PLANE)?; }
    }
    Ok(())
}
```

פונקציה זו ממחישה Composition במקום ירושה: `channel_method` מועבר כפרמטר גנרי. הפונקציה לא יודעת ולא אכפת לה אם מדובר בהטמעה או בחילוץ — היא רק מנתבת לכל ערוץ שנבחר ב-configs. הפרמטרים הגנריים `T` ו-`W` מאפשרים שימוש בפונקציה עם מצבי state שונים לגמרי.

`find_dimensions_for_codec` מחזירה ממדים שונים לפי פורמט הפיקסל: YUV420P מחזיק ערוצי U ו-V ברבע הגודל (חצי בכל ממד), YUV444P מחזיק את כולם בגודל מלא. הטמעה בערוץ U בממדים שגויים תשבש את מבנה הפריים.

---

#### 21.5.8 — embed_in_channel: לב ההטמעה

**קובץ:** `steganography_service/src/services/embed_video.rs`

זוהי הפונקציה המורכבת ביותר במערכת.

**סינון I-frames:**
```rust
if !frame.is_key() {
    return Ok(());
}
```

רק פריימי מפתח עצמאיים מעובדים, כפי שהוסבר בסעיף 15.1.

**חישוב block_offset עם stride:**
```rust
pub const BLOCKS_PER_MACROBLOCK: u32 = 4;

let block_offset = 4 * block_row * stride as u32 * BLOCKS_PER_MACROBLOCK
                 + 4 * block_col * BLOCKS_PER_MACROBLOCK;
```

`BLOCKS_PER_MACROBLOCK = 4` פירושו צעד של 16 פיקסלים — גבול ה-macroblock המקסימלי של H.264. הכפל ב-`stride` הכרחי כי שורה אחת בנתוני הפריים עשויה להיות רחבה יותר מרוחב הפריים בגלל ריפוד ליישור זיכרון.

**קריאת הבלוק תוך שמירה על stride:**
```rust
for i in 0..4 {
    for j in 0..4 {
        block_as_pixel[i * 4 + j] = frame_data[block_offset as usize + i * stride + j];
    }
}
let block_as_dct = dct_ii(&block_as_pixel);
```

**מנגנון הצבירה עם HashMap:**
```rust
state.pending_blocks.insert(block_offset, PendingBlock {
    coeffs: block_as_dct,
    coeffs_left_to_embed: state.coeffs_to_embed_count_block,
});

for i in 0..16 {
    if !configs.coefficients_to_embed[i] { continue; }

    state.coeff_accumulator_pos.push((block_offset, i as u8));
    state.pending_blocks.get_mut(&block_offset)?.coeffs_left_to_embed -= 1;

    if state.coeff_accumulator_pos.len() >= configs.coefficients_per_bit {
        embed_bit_in_coefficients(state, payload_buffer, configs)?;

        state.pending_blocks.retain(|offset, block| {
            if block.coeffs_left_to_embed == 0 {
                apply_modified_dct_coeffs_on_frame(&block.coeffs, frame_data, *offset, stride);
                return false;
            }
            true
        });
        state.coeff_accumulator_pos.clear();
    }
}
```

מנגנון "חלון נגלל": המצבר צובר מקדמים מבלוקים שונים עד שנצברו `coefficients_per_bit` מקדמים. רק אז מוטמע ביט אחד. בלוקים שכל מקדמיהם כבר שויכו — מוחלים חזרה לפריים ומוסרים מה-HashMap. זה מאפשר לשיטות כמו SS לשתף מקדמים מבלוקים שונים לייצוג ביט אחד.

---

#### 21.5.9 — embed_bit_in_coefficients: הטריק של ה-raw pointer

**קובץ:** `steganography_service/src/services/embed_video.rs`

```rust
fn embed_bit_in_coefficients(
    state: &mut EmbedState,
    payload_buffer: &mut PayloadBuffer,
    configs: &EmbedConfigs,
) -> Result<(), StegServiceError> {
    if state.payload_exhausted {
        return Ok(());
    }
    let target_bit: bool;
    if payload_buffer.bit_index >= payload_buffer.bits_read {
        populate_payload_buffer(state, payload_buffer)?;
        if state.payload_exhausted {
            return Ok(());
        }
    }

    let target_byte = payload_buffer.buffer[payload_buffer.bit_index / 8];
    target_bit = (target_byte >> (7 - (payload_buffer.bit_index % 8))) & 0x1 == 0x1;
    payload_buffer.bit_index += 1;

    let state_ptr = state as *mut EmbedState;

    configs.method.embed(
        |i| get_coeff(state_ptr, i),
        |i, v| set_coeff(state_ptr, i, v),
        configs.coefficients_per_bit,
        configs.seed.clone(),
        target_bit,
        configs.delta,
    )?;
    Ok(())
}
```

**מדוע unsafe?** `method.embed` מקבל שני closures: `get_coeff` ו-`set_coeff`. שניהם צריכים גישה ל-`state`. בקוד רגיל, Rust's borrow checker לא מאפשר שני borrows לאותו struct בו-זמנית — גם אם בפועל הם אינם פועלים בו-זמנית. המרה ל-raw pointer עוקפת הגבלה זו.

הקוד בטוח כי:
1. השרשרת כולה synchronous — שרשור אחד.
2. `get_coeff` ו-`set_coeff` אף פעם לא נקראות בו-זמנית; תמיד תחילה `get` ואז `set`.

---

#### 21.5.10 — write_bit_to_payload_buffer: הרכבת בתים ופענוח כותרת

**קובץ:** `steganography_service/src/services/extract_file.rs`

```rust
fn write_bit_to_payload_buffer(
    target_bit: bool,
    buffer: &mut PayloadBuffer,
    state: &mut ExtractState,
) -> Result<(), StegServiceError> {
    // assemble bits MSB first
    buffer.buffer[buffer.bit_index / 8] <<= 1;
    if target_bit {
        buffer.buffer[buffer.bit_index / 8] |= 0x1;
    }
    buffer.bit_index += 1;

    // header parse — exactly once, when 64 bits have been accumulated
    if state.payload_size == 0 && buffer.bit_index == HEADER_SIZE_BITS {
        state.payload_size = u64::from_le_bytes(
            buffer.buffer[0..8].try_into().map_err(|_| StegServiceError::FileError)?,
        );
        buffer.bit_index = 0;
        buffer.buffer = [0; 1028];
        return Ok(());
    }

    // early stop check after every complete byte
    if state.payload_size > 0 && buffer.bit_index % 8 == 0 {
        let total = state.total_extracted_bytes + (buffer.bit_index as u64 / 8);
        if total >= state.payload_size {
            buffer.writer.write_all(&buffer.buffer[0..(buffer.bit_index / 8)])?;
            state.extraction_ongoing = false;
            return Ok(());
        }
    }

    // flush buffer when full
    if buffer.bit_index >= buffer.buffer.len() * 8 {
        buffer.writer.write_all(&buffer.buffer[0..buffer.bit_index / 8])?;
        state.total_extracted_bytes += buffer.bit_index as u64 / 8;
        buffer.bit_index = 0;
        buffer.buffer = [0; 1028];
    }
    Ok(())
}
```

שלוש החלטות הנדסיות מעניינות:

1. **MSB ראשון:** `<<= 1` מזיז שמאלה ואז `|= 0x1` שם את הביט בעמדה הנמוכה. זה מבטיח סדר עקבי עם צד ההטמעה.

2. **פענוח כותרת אינטגרלי:** אין מצב מיוחד "אנחנו בכותרת". הפונקציה בודקת: אם גודל ה-payload עדיין לא ידוע ועכשיו נצברו בדיוק 64 ביטים — זו הכותרת. הבאפר מאופס והחילוץ האמיתי מתחיל.

3. **עצירה מוקדמת:** בכל בית מושלם נבדק אם הגענו לגודל המטען הצפוי. ללא זה, החילוץ היה ממשיך עד סוף הווידאו וממלא את הפלט בנתוני "זבל".

---

#### 21.5.11 — embed_video route: מקביליות ועבודה חוסמת

**קובץ:** `steganography_service/src/routes/embed_video.rs`

```rust
pub async fn embed_video(
    State(app_state): State<AppState>,
    AuthenticatedUserWithToken(user, access_token): AuthenticatedUserWithToken,
    Json(payload): Json<EmbedFileRequest>,
) -> Result<(StatusCode, Json<FileResponse>), StegServiceError> {
    tracing::info!("User with id: {} attempting to embed video", user);
    let files_service_url = app_state
        .eureka_config
        .read()
        .unwrap()
        .services
        .get("files_service")
        .ok_or(StegServiceError::EurekaConfigError)?
        .to_string();

    // download carrier and payload concurrently
    let ((carrier_path, is_valid, _, carrier_filename), (payload_path, _, _, payload_filename)) =
        tokio::try_join!(
            files_client::download_file_to_temp(
                &app_state.client,
                &files_service_url,
                payload.carrier_id,
                &access_token,
            ),
            files_client::download_file_to_temp(
                &app_state.client,
                &files_service_url,
                payload.payload_id,
                &access_token,
            )
        )?;
    if !is_valid {
        tracing::error!("Invalid payload for user: {}", user);
        return Err(StegServiceError::InvalidPayload);
    }

    let payload_path_clone = payload_path.clone();
    let carrier_path_clone = carrier_path.clone();

    // embed on a dedicated thread — CPU-bound work
    let output_path = tokio::task::spawn_blocking(move || {
        embed(payload_path_clone, carrier_path_clone, payload.configs)
    })
    .await
    .map_err(|_| StegServiceError::Other("embed task panicked".to_string()))??;

    let steg_file_remote_pointer = files_client::upload_file_to_files_service(
        payload_path,
        carrier_path,
        output_path,
        payload_filename,
        carrier_filename,
        &app_state.client,
        &files_service_url,
        &access_token,
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to upload file to files service");
        e
    })?;

    tracing::info!(
        "Steganography pipeline complete for user: {}",
        user
    );

    Ok((StatusCode::CREATED, Json(steg_file_remote_pointer)))
}
```

**`tokio::try_join!`:** הורדת שני קבצים בצורה סדרתית תיקח `t1 + t2` שניות. `try_join!` מריץ את שתי הפעולות בו-זמנית והזמן הכולל הוא `max(t1, t2)`. אם אחת נכשלת — השנייה מבוטלת מיד.

**`spawn_blocking`:** ה-Tokio runtime מנהל thread-pool קטן. פעולות CPU-intensive כמו ההטמעה חוסמות thread ורעבות tasks אחרים. `spawn_blocking` מעביר את הפעולה ל-thread pool נפרד המיועד לעבודה חוסמת. ה-`??` מפרק שתי שכבות של Result: אחת מ-`spawn_blocking` (JoinError אם ה-thread נקרס) ואחת מ-`embed` עצמה.

---

#### 21.5.12 — חשבון שדה גאלואה GF(2⁸) וטבלאות חיפוש

**קובץ:** `steganography_service/src/services/gause_field.rs`

**כפל בשתיים עם טיפול ב-overflow:**
```rust
const POLYNOMAL_PRIME_NUMBER: u16 = 0x11d;

pub fn polynomal_multiplication_by_two(mut num: u8) -> u8 {
    if num & 0b10000000 == 0b10000000 {
        num <<= 1;
        num = (num as u16 ^ POLYNOMAL_PRIME_NUMBER) as u8;
        return num;
    }
    num <<= 1;
    return num;
}
```

בשדה GF(2⁸), כפל בשתיים הוא shift left, אך כשה-MSB דלוק ה-shift יוצר overflow. הפתרון: XOR עם הפולינום הלא-פריק `0x11D = x⁸ + x⁴ + x³ + x² + 1`, שמשמר את תכונות האלגברה בשדה.

**טבלאות חיפוש עם LazyLock:**
```rust
pub static EXP_TABLE: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let mut exp_table: Vec<u8> = Vec::with_capacity(256);
    let mut value: u8 = 1;
    exp_table.push(1);
    for _ in 1..256 {
        value = polynomal_multiplication_by_two(value);
        exp_table.push(value);
    }
    exp_table
});

pub static LOG_TABLE: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let mut log_table = vec![0u8; 256];
    for i in 0..256 {
        log_table[EXP_TABLE[i] as usize] = i as u8;
    }
    log_table
});
```

`LazyLock` מאתחל את הטבלה בפעם הראשונה שנגשים אליה, בצורה thread-safe, ושומר את התוצאה לתמיד. הטבלאות נבנות פעם אחת בלבד לכל אורך חיי התהליך.

**כפל כללי — O(1) במקום O(log n):**
```rust
pub fn poly_mult(num1: u8, num2: u8) -> u8 {
    if num1 == 0 || num2 == 0 { return 0; }
    let mut index_sum =
        (LOG_TABLE[num1 as usize] + LOG_TABLE[num2 as usize]) as u16;
    while index_sum > 255 { index_sum -= 255; }
    EXP_TABLE[index_sum as usize]
}
```

`log(a) + log(b) = log(a·b)` — בשדה GF(2⁸) החיבור מודולרי mod 255. שתי גישות לטבלה ופעולת חיבור אחת במקום כפל חוזר.

---

#### 21.5.13 — Reed-Solomon: קידוד

**קבצים:** `steganography_service/src/services/reed_solomon.rs` ו-`rs_generator_vec.rs`

**בניית פולינום המחולל:**
```rust
pub fn generatae_generator(len: u8) -> Vec<u8> {
    let mut generator: Vec<u8> = vec![1, 1]; // (x + 2^0)

    for i in 1..len {
        let mut root = vec![1; 2];
        root[0] = EXP_TABLE[i as usize]; // (x + 2^i)
        generator = poly_mult_vecs(&generator, &root);
    }
    generator
}
```

פולינום המחולל הוא `g(x) = (x - 2⁰)(x - 2¹)···(x - 2^(len-1))`. שורשיו הם הנקודות שבהן `g(x) = 0`, מה שמאפשר גילוי שגיאות על ידי הצבתן בפענוח.

**קידוד chunk:**
```rust
fn encode_chunk(chunk: &mut Vec<u8>, generator: &[u8]) -> Result<(), StegServiceError> {
    let chunk_len = chunk.len();
    chunk.append(&mut vec![0u8; generator.len() - 1]); // append zero padding
    let remainder = poly_div_remainder_vecs(&chunk, generator); // polynomial long division
    for i in 0..remainder.len() {
        chunk[chunk_len + i] = remainder[i]; // replace zeros with remainder (parity bytes)
    }
    Ok(())
}
```

מוסיפים `P` אפסים לקצה ה-chunk, מחלקים בפולינום המחולל, ומחליפים את האפסים בשארית. המסר המקודד (data + parity) מתחלק בפולינום המחולל ללא שארית — תכונה זו מאפשרת גילוי שגיאות בפענוח.

**TODO — פענוח Reed-Solomon:** חישוב ה-Syndromes מומש. שלב הפענוח המלא — Berlekamp-Massey לאיתור מיקומי השגיאות, חישוב עוצמתן ותיקון הבתים — טרם הושלם ומסומן כ-`todo!()`.

---

## 21.6 ה-API Gateway

### פונקציות מרכזיות

---

#### 21.6.1 — proxy_request: Reverse Proxy ללא באפר

**קובץ:** `api_gateway/src/proxy.rs`

```rust
pub async fn proxy_request(service_url: &str, req: Request) -> Result<Response, StatusCode> {
    let method = req.method().clone();
    let stripped_path = req.uri().path_and_query()
        .map(|pq| pq.as_str()).unwrap_or("/")
        .strip_prefix("/api").unwrap_or("/");
    let headers = req.headers().clone();
    let url = format!("{}{}", service_url, stripped_path);

    let backend_response = reqwest::Client::new()
        .request(method, &url)
        .headers(headers)
        .body(reqwest::Body::wrap_stream(req.into_body().into_data_stream()))
        .send().await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let mut response = Response::builder().status(backend_response.status());
    for (key, value) in backend_response.headers() {
        response = response.header(key, value);
    }
    response
        .body(Body::from_stream(backend_response.bytes_stream()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
```

**Streaming zero-copy:** גוף הבקשה הנכנסת `req.into_body().into_data_stream()` עטוף ב-`reqwest::Body::wrap_stream` ונשלח ישירות לשירות הפנימי. גוף התגובה `backend_response.bytes_stream()` מוחזר ישירות ללקוח. כל הגוף אינו עולה לזיכרון בבת אחת — קריטי עבור קבצי וידאו גדולים.

**כתובות שירותים דינמיות:** ה-handlers (כגון `auth_handler`, `files_handler`) קוראים את כתובת השירות מ-`state.eureka_configs.read().unwrap().services.get("...")`. ה-`RwLock` מאפשר קריאות מקביליות רבות, ומתעדכן כל 30 שניות על ידי background task — כך ה-Gateway תמיד מנתב לכתובת העדכנית ללא restart.
