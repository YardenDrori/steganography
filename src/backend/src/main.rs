use image::ImageReader;

const IMAGE_FILE: &str = "../../examples/images/png_image.png";

fn main() {
    let frame = image::ImageReader::open(IMAGE_FILE). //there are like 30 unwrap options also it says in the docs to use "let img = ImageReader::open("myimage.png")?.decode()?;";
    let dimensions = frame //broooooooooooooooooooooo wtf is this
}
