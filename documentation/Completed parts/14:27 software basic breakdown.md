# תיעוד פרויקט - פלטפורמת סטגנוגרפיה בווידאו

---

## תיאור התוכנה

### 14.1 פירוט API

כל הנתיבים עוברים דרך ה-API Gateway בכתובת הבסיס `/api`.

עמודת "אימות" בטבלאות: `JWT` — Bearer token בכותרת `Authorization`; `Admin` — JWT עם תפקיד מנהל; `Cookie` — refresh_token ב-HttpOnly Cookie.

---

#### שירות האימות — `/api/auth/`

| Method | נתיב | תיאור | אימות | קוד תגובה |
|--------|------|--------|--------|------------|
| `POST` | `/api/auth/register` | הרשמת משתמש חדש | ללא | 201 |
| `POST` | `/api/auth/login` | התחברות | ללא | 200 |
| `POST` | `/api/auth/refresh` | חידוש access token | Cookie | 200 |
| `POST` | `/api/auth/logout` | התנתקות | Cookie | 204 |
| `POST` | `/api/auth/deactivate` | השבתת חשבון עצמי | JWT | 204 |
| `PATCH` | `/api/auth/admin/users/:id/activate` | הפעלת חשבון משתמש | Admin | 204 |
| `PATCH` | `/api/auth/admin/users/:id/deactivate` | השבתת חשבון משתמש | Admin | 204 |

**`POST /api/auth/register`**

גוף הבקשה:
```json
{
  "user_name": "string (3–50 תווים)",
  "email": "string (כתובת מייל תקנית)",
  "password": "string (8–128 תווים)",
  "first_name": "string (1–50 תווים)",
  "last_name": "string (1–50 תווים)",
  "phone_number": "string | null",
  "is_male": "boolean | null"
}
```

תגובה (201):
```json
{
  "user": { "id": 1, "user_name": "...", "first_name": "...", "last_name": "...",
            "email": "...", "phone_number": null, "is_male": null,
            "is_verified": false, "created_at": "...", "updated_at": "..." },
  "access_token": "eyJ..."
}
```
Set-Cookie: `refresh_token=<token>; HttpOnly; SameSite=Strict`.

---

**`POST /api/auth/login`** — נדרש אחד מ-`email` / `user_name`.

```json
{
  "email": "string | null",
  "user_name": "string | null",
  "password": "string",
  "device_info": "string | null"
}
```

תגובה: זהה ל-register, קוד 200.

---

**`POST /api/auth/refresh`** — אין גוף בקשה. ה-Cookie נשלח אוטומטית.

תגובה (200):
```json
{ "access_token": "eyJ..." }
```
Set-Cookie: refresh token חדש (הישן מבוטל).

---

#### שירות המשתמשים — `/api/users/`

| Method | נתיב | תיאור | אימות | קוד תגובה |
|--------|------|--------|--------|------------|
| `GET` | `/api/users/me` | קבלת פרופיל עצמי | JWT | 200 |
| `PATCH` | `/api/users/me` | עדכון פרופיל עצמי | JWT | 200 |
| `GET` | `/api/users/:id` | קבלת פרופיל לפי מזהה | Admin | 200 |
| `PATCH` | `/api/users/:id` | עדכון משתמש | Admin | 200 |
| `DELETE` | `/api/users/:id` | מחיקת משתמש | Admin | 204 |

**`PATCH /api/users/me`** — כל השדות אופציונליים, שולחים רק מה שרוצים לעדכן:
```json
{
  "user_name": "string | null",
  "first_name": "string | null",
  "last_name": "string | null",
  "email": "string | null",
  "phone_number": "string | null",
  "is_male": "boolean | null"
}
```
תגובה (200):
```json
{
  "id": 1,
  "user_name": "string",
  "first_name": "string",
  "last_name": "string",
  "email": "string",
  "phone_number": "string | null",
  "is_male": "boolean | null",
  "is_verified": false,
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T00:00:00Z"
}
```

---

#### שירות הקבצים — `/api/files/`

| Method | נתיב | תיאור | אימות | קוד תגובה |
|--------|------|--------|--------|------------|
| `POST` | `/api/files/initiate` | פתיחת multipart upload | JWT | 200 |
| `POST` | `/api/files/upload-chunk` | העלאת chunk בודד | JWT | 200 |
| `POST` | `/api/files/complete` | סיום upload ורישום קובץ | JWT | 201 |
| `GET` | `/api/files/me` | רשימת קבצי המשתמש | JWT | 200 |
| `GET` | `/api/files/:id` | מטא-דאטה של קובץ | JWT | 200 |
| `GET` | `/api/files/:id/download` | הורדת קובץ | JWT | 200 |
| `PATCH` | `/api/files/:id` | שינוי שם קובץ | JWT | 200 |
| `DELETE` | `/api/files/:id` | מחיקת קובץ | JWT | 204 |

**`POST /api/files/initiate`** — אין גוף בקשה. מחזיר `upload_id` ו-`object_key` לשימוש בשלבים הבאים.

תגובה (200):
```json
{ "upload_id": "string", "object_key": "string" }
```

---

**`POST /api/files/upload-chunk`** — גוף הבקשה הוא bytes גולמיים (binary). הפרמטרים מועברים ב-query string:

```
POST /api/files/upload-chunk?part_number=1&upload_id=...&object_key=...
```

תגובה (200):
```json
{ "part": { "part_number": 1, "etag": "string" } }
```

---

**`POST /api/files/complete`**

גוף הבקשה:
```json
{
  "upload_id": "string",
  "object_key": "string",
  "filename": "string",
  "parts": [
    { "part_number": 1, "etag": "string" }
  ]
}
```

תגובה (201):
```json
{
  "id": 42,
  "filename": "video.mp4",
  "created_at": "2025-01-01T00:00:00Z",
  "is_carrier": false,
  "is_steg_object": false
}
```

---

#### שירות הסטגנוגרפיה — `/api/embed/` ו-`/api/extract/`

| Method | נתיב | תיאור | אימות | קוד תגובה |
|--------|------|--------|--------|------------|
| `POST` | `/api/embed/video` | הטמעת מטען בווידאו | JWT | 201 |
| `POST` | `/api/extract/video` | חילוץ מטען מווידאו | JWT | 200 |

**`POST /api/embed/video`**

גוף הבקשה:
```json
{
  "carrier_id": 10,
  "payload_id": 11,
  "configs": {
    "channels_to_embed": {
      "yuv": { "y": true, "cb": false, "cr": false },
      "rgb": null
    },
    "coefficients_to_embed": [false, false, false, false, false, true, true, true, false, false, false, false, false, false, false, false],
    "coefficients_per_bit": 2,
    "blocks_per_macroblock": 1,
    "delta": 120,
    "seed": "my-secret-seed",
    "method": "ISS",
    "reed_solomon_padding_byte_count": 16,
    "aes_password": "string | null"
  }
}
```

`method` מקבל: `QIM`, `STDM`, `SS`, `ISS`. תגובה (201): `FileResponse` של הווידאו שנוצר.

---

**`POST /api/extract/video`**

גוף הבקשה:
```json
{
  "steg_object_id": 42,
  "configs": { ... }
}
```

ה-`configs` חייב להיות זהה לחלוטין לזה שבו בוצעה ההטמעה, אחרת החילוץ ייכשל. תגובה (200): `FileResponse` של המטען המחולץ.

---

### 14.2 סביבת עבודה

**מערכות הפעלה**

הפרויקט פותח על שני מחשבים. הראשון מריץ macOS 16 Tahoe, שנבחר בשל הבסיס הלינוקסי שלו ותאימותו הטבעית לכלי פיתוח, ובמיוחד בשל ביצועי המעבד. השני מריץ Arch Linux, שנבחר בשל גמישותו המלאה ומדיניות השחרור הגלגלתית שלו, המבטיחה גישה לגרסאות העדכניות ביותר של כל חבילה ללא תלות במחזורי שחרור קבועים.

**Neovim**

כלי הפיתוח העיקרי בשתי הסביבות הוא Neovim. Neovim הוא עורך טקסט מבוסס מסוף, בשונה מ-IDE כבד כמו VSCode או IntelliJ הוא פועל ישירות במסוף ואינו מצריך ממשק גרפי. הוא מספק יכולות ניתוח קוד, השלמה אוטומטית ואבחון שגיאות בזמן אמת דרך פרוטוקול LSP ותוספים ייעודיים לכל שפה, תוך שימוש מינימלי במשאבים. הגדרה אחת של Neovim עובדת זהה בשתי הסביבות ללא שינוי.

**Git**

Git הוא מערכת בקרת גרסאות מבוזרת. הוא מאפשר לשמור היסטוריה מלאה של שינויים בקוד, לחזור לכל נקודה בזמן, ולעבוד במקביל על ענפים שונים של הפרויקט. Git שימש לניהול כל שינוי בקוד לאורך הפרויקט.

**GitHub**

GitHub היא פלטפורמת אחסון ושיתוף קוד מבוססת Git. היא מספקת ממשק ויזואלי לניהול הריפוזיטורי, מעקב אחר שינויים, וניהול גרסאות. הפרויקט מתארח ב-GitHub ושימש כנקודת סנכרון בין שני מחשבי הפיתוח.

**Postman**

Postman הוא כלי לבדיקת API. הוא מאפשר לשלוח בקשות HTTP מכל סוג לנתיבי ה-API, לראות את התשובות בפורמט קריא, ולשמור אוספי בקשות לשימוש חוזר. השתמשנו בו לאורך הפיתוח לבדיקת כל נתיב לפני בניית ממשק המשתמש.

**Docker**

Docker הוא כלי לאריזת תוכנה בתוך containers, יחידות מבודדות שמכילות את התוכנה וכל תלויותיה. במקום להתקין PostgreSQL, MinIO ושאר השירותים ישירות על המחשב, כל אחד מהם רץ בתוך container נפרד. זה מבטיח שהסביבה זהה בין שני מחשבי הפיתוח ומפשט את ההרצה של כל השירותים יחד.

**כלי Rust**

שרשרת כלי Rust מנוהלת דרך `rustup`, מנהל גרסאות ל-Rust המאפשר התקנה ועדכון של המהדר בקלות. `cargo` משמש כמנהל הפרויקט והתלויות, ומטפל בקומפילציה, הרצת בדיקות והורדת ספריות חיצוניות. `sqlx-cli` הוא כלי שורת פקודה לניהול מיגרציות מסד הנתונים.

**Node.js ו-npm**

Node.js היא סביבת ריצה ל-JavaScript מחוץ לדפדפן. npm הוא מנהל החבילות שמגיע איתה. השתמשנו בהם לבניית צד ה-frontend והרצת שרת הפיתוח.

**FFmpeg**

FFmpeg הוא ספריית עיבוד מולטימדיה בקוד פתוח המאפשרת קריאה, כתיבה והמרה של קבצי וידאו ואודיו. השתמשנו בה דרך bindings ל-Rust לצורך חילוץ פריימים מהוידאו, עיבודם, וכתיבת הפלט חזרה לקובץ וידאו.

---

### 14.3 שפות תכנות

הפרויקט משלב מספר שפות ומסגרות עבודה בהתאם לדרישות כל שכבה.

**Rust** משמשת לכלל שירותי ה-backend. היא נבחרה בשל ביצועיה הגבוהים, ניהול זיכרון ללא garbage collector, ומערכת הטיפוסים החזקה שלה שמונעת מחלקה גדולה מהבאגים בזמן קומפילציה. מסגרת Axum משמשת לבניית שרתי HTTP, ו-SQLx לגישה בטוחת טיפוסים למסד הנתונים.

**TypeScript עם React 19** משמשת לממשק המשתמש. TypeScript נבחרה על פני JavaScript כדי להוסיף בטיחות טיפוסים גם בצד הלקוח. Tailwind CSS v4 משמש לעיצוב.

**SQL** משמש להגדרת סכמת מסד הנתונים ולהרצת מיגרציות דרך SQLx.
