use clap::{App, Arg};
use image::{GenericImageView, ImageBuffer};

const TILE_OFFSETS: [(u32, u32); 6] = [(2, 1), (0, 1), (1, 0), (1, 2), (1, 1), (3, 1)];

fn main() {
    let matches = App::new("Cubemap Stitcher")
        .version("1.0")
        .author("Zander F. <zander@zanderf.net>")
        .about("Stitches cubemap images together into a cubemap atlas")
        .arg(
            Arg::with_name("XPOS")
                .help("The x-positive image file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("XNEG")
                .help("The x-negative image file")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("YPOS")
                .help("The y-positive image file")
                .required(true)
                .index(3),
        )
        .arg(
            Arg::with_name("YNEG")
                .help("The y-negative image file")
                .required(true)
                .index(4),
        )
        .arg(
            Arg::with_name("ZPOS")
                .help("The z-positive image file")
                .required(true)
                .index(5),
        )
        .arg(
            Arg::with_name("ZNEG")
                .help("The z-negative image file")
                .required(true)
                .index(6),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .default_value("cubemap.png"),
        )
        .get_matches();

    let images = [
        image::open(matches.value_of("XPOS").unwrap()).expect("Failed to find x-positive image"),
        image::open(matches.value_of("XNEG").unwrap()).expect("Failed to find x-negative image"),
        image::open(matches.value_of("YPOS").unwrap()).expect("Failed to find y-positive image"),
        image::open(matches.value_of("YNEG").unwrap()).expect("Failed to find y-negative image"),
        image::open(matches.value_of("ZPOS").unwrap()).expect("Failed to find z-positive image"),
        image::open(matches.value_of("ZNEG").unwrap()).expect("Failed to find z-negative image"),
    ];

    // assert that every image has the same width and height
    let mut iwh_iter = images.iter();
    let iwh_first = iwh_iter.next().unwrap();
    let (iwh_w, iwh_h) = (iwh_first.width(), iwh_first.height());
    assert!(
        iwh_w == iwh_h,
        "The width and height must be the same for each tile"
    );
    for iwh_img in iwh_iter {
        assert!(
            iwh_img.width() == iwh_w,
            "Not all images have the same width/height"
        );
        assert!(
            iwh_img.height() == iwh_h,
            "Not all images have the same width/height"
        );
    }

    // let's make a new image and stitch these together
    let mut imgbuf = ImageBuffer::new(iwh_w * 4, iwh_h * 3);

    for (idx, image) in images.iter().enumerate() {
        let offset = TILE_OFFSETS[idx];
        for y in 0..iwh_h {
            for x in 0..iwh_w {
                imgbuf.put_pixel(
                    x + offset.0 * iwh_w,
                    y + offset.1 * iwh_h,
                    image.get_pixel(x, y),
                );
            }
        }
    }

    // write it out
    imgbuf.save(matches.value_of("output").unwrap()).expect("Failed to save cubemap atlas");
}
