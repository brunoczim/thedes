use crate::color::{
    pair::MutationExt,
    ApproxBrightness,
    Brightness,
    ColorPair,
    ContrastFgWithBg,
    GrayColor,
    LegacyRgb,
    Mutation,
    RgbColor,
    SetBg,
    SetFg,
};

#[test]
fn gray_color_brightness() {
    assert_eq!(GrayColor::new(0).approx_brightness(), Brightness { level: 0 });
    assert_eq!(
        GrayColor::new(3).approx_brightness(),
        Brightness { level: 8548 }
    );
    assert_eq!(
        GrayColor::new(9).approx_brightness(),
        Brightness { level: 25644 }
    );
    assert_eq!(
        GrayColor::new(12).approx_brightness(),
        Brightness { level: 34192 }
    );
    assert_eq!(
        GrayColor::new(23).approx_brightness(),
        Brightness { level: 65535 }
    );
}

#[test]
fn cmy_color_brightness() {
    assert_eq!(
        LegacyRgb::new(0, 0, 0).approx_brightness(),
        Brightness { level: 0 }
    );
    assert_eq!(
        LegacyRgb::new(1, 2, 3).approx_brightness(),
        Brightness { level: 26214 }
    );
    assert_eq!(
        LegacyRgb::new(5, 5, 5).approx_brightness(),
        Brightness { level: 65535 }
    );
}

#[test]
fn rgb_color_brightness() {
    assert_eq!(
        RgbColor { red: 0, green: 0, blue: 0 }.approx_brightness(),
        Brightness { level: 0 }
    );
    assert_eq!(
        RgbColor { red: 51, green: 102, blue: 153 }.approx_brightness(),
        Brightness { level: 23644 }
    );
    assert_eq!(
        RgbColor { red: 255, green: 255, blue: 255 }.approx_brightness(),
        Brightness { level: 65535 }
    );
}

#[test]
fn mutators() {
    let mutation = SetFg(LegacyRgb::new(1, 2, 3).into())
        .then(ContrastFgWithBg)
        .then(SetBg(LegacyRgb::new(4, 4, 5).into()));

    let pair = ColorPair {
        foreground: RgbColor { red: 0, green: 0, blue: 0 }.into(),
        background: LegacyRgb::new(1, 1, 1).into(),
    };

    assert_eq!(
        mutation.mutate_colors(pair),
        ColorPair {
            foreground: LegacyRgb::new(2, 3, 5).into(),
            background: LegacyRgb::new(4, 4, 5).into(),
        }
    );
}
