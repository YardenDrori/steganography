use std::f64::consts::PI;

pub fn dct_ii(pixels: [[u8; 4]; 4]) -> [[f64; 4]; 4] {
    let mut coefficients = [[0.0f64; 4]; 4];

    //to understand this weird ass forumula what i recommend is going to desmos and pasting these
    //values as 1 function and 4 positions
    //y=\cos\left(kx\right)
    //\left(\frac{\left(2i+o\right)}{8}\pi,\cos\left(k\left(\frac{\left(\left(2i+o\right)\pi\right)}{8}\right)\right)\right)
    //\left(\frac{\left(2\left(i+1\right)+o\right)}{8}\pi,\cos\left(k\left(\frac{\left(\left(2\left(i+1\right)+o\right)\pi\right)}{8}\right)\right)\right)
    //\left(\frac{2\left(i+2\right)+o}{8}\pi,\cos\left(k\left(\frac{\left(\left(2\left(i+2\right)+o\right)\pi\right)}{8}\right)\right)\right)
    //\left(\frac{\left(2\left(i+3\right)+o\right)}{8}\pi,\cos\left(k\left(\frac{\left(\left(2\left(i+3\right)+o\right)\pi\right)}{8}\right)\right)\right)
    //where i is a value between 0 and 3 with jump size of 1
    //where k is a value between 0 and 3 with jump size of 1
    //where o is either 0 or 1 to see the difference between dct and dctII (what we're doing)
    for k_x in 0..4 {
        for k_y in 0..4 {
            let mut sum = 0.0f64;
            for i_x in 0..4 {
                for i_y in 0..4 {
                    let curr_pixel = pixels[i_x][i_y] as f64;
                    let angle_x = k_x as f64 * (2.0 * i_x as f64 + 1.0) * PI / 8.0;
                    let angle_y = k_y as f64 * (2.0 * i_y as f64 + 1.0) * PI / 8.0;
                    sum += curr_pixel * angle_x.cos() * angle_y.cos();
                }
            }
            coefficients[k_x][k_y] = sum;
        }
    }
    coefficients
}

/*this method reverses dct coefficients back to pixel values by taking the original formula where C
is coefficient, V is the value from the whole COS formula and P is the pixel value so in the
original formula is is C1 = P1*V1+P2+V2+P3+V3+P4+V4, and we swap the coefficient with the pixel
so the new formula will be P1 = C1*V1+C2*V2+C3*V3+C4*V4 but unfortunately due to linear algebra
being an absolute bitch we dont get the original values the values for k=0 will be 4x as big and
for all other k values they will be 2x i dont really get why thats happening but i ran the
numbers and it is happening so this code handles it lol we also clamp and round the numbers so
that float point errors dont add up and that if we hadd a dct value of 100 and we made it 102 the
cascading change on all pixels the dct influenced wont push them to a negative value which will
be interperted as 255 when converting them back to u8*/
pub fn idct_ii(coefficients: [[f64; 4]; 4]) -> [[u8; 4]; 4] {
    let mut pixels = [[0u8; 4]; 4];

    for i_x in 0..4 {
        for i_y in 0..4 {
            let mut sum = 0.0f64;
            for k_x in 0..4 {
                for k_y in 0..4 {
                    let curr_coefficient = coefficients[k_x][k_y];
                    let scale_x = if k_x == 0 { 0.25 } else { 0.5 };
                    let scale_y = if k_y == 0 { 0.25 } else { 0.5 };
                    let angle_x = k_x as f64 * (2.0 * i_x as f64 + 1.0) * PI / 8.0;
                    let angle_y = k_y as f64 * (2.0 * i_y as f64 + 1.0) * PI / 8.0;
                    sum += curr_coefficient * scale_x * angle_x.cos() * scale_y * angle_y.cos();
                }
            }
            pixels[i_x][i_y] = sum.round().clamp(0.0, 255.0) as u8;
        }
    }
    pixels
}
