extern crate lua_patterns;
use lua_patterns::LuaPattern;
use std::vec;

fn parse_f32(s: (&str, &str)) -> Option<f32> {
    let chapter = s.0.parse::<f32>().unwrap();

    if s.1.is_empty() {
        return Some(chapter);
    }

    let subchapter = if s.1.contains("extra") {
        0.99
    } else if s.1.contains("omake") {
        0.98
    } else if s.1.contains("special") {
        0.97
    } else if let Some(chr) = s.1.chars().next() {
        if let Ok(num) = s.1.parse::<f32>() {
            num / (10_u32.pow(s.1.len() as u32) as f32)
        } else {
            match chr {
                'a'..='h' => (chr as u8 as f32 - 96.0) / 10.0,
                // ASCII 65 to 74
                'A'..='H' => (chr as u8 as f32 - 64.0) / 10.0,
                _ => 0.0,
            }
        }
    } else {
        0.0
    };

    Some(chapter + subchapter)
}

pub fn parse_chapter_number(title: String, chapter: String) -> f32 {
    let mut number_pattern = LuaPattern::new("(%d+)%.?(%w*)");
    let mut basic_pattern = LuaPattern::new("ch.%s-(%d+)%.?(%w*)");

    let unwanted_patterns = vec![
        LuaPattern::new("%f[%a][vs]%A?%d+"),
        LuaPattern::new("%f[%a]ver%A?%d+"),
        LuaPattern::new("%f[%a]vol%A?%d+"),
        LuaPattern::new("%f[%a]version%A?%d+"),
        LuaPattern::new("%f[%a]volume%A?%d+"),
        LuaPattern::new("%f[%a]season%A?%d+"),
    ];

    let replacements = vec![
        (" special", ".special"),
        (" omake", ".omake"),
        (" extra", ".extra"),
    ];

    let mut name = chapter.to_lowercase();

    name = name.replace(&title.to_lowercase(), "").trim().to_string();

    name = name.replace([',', '-'], ".");

    for mut pattern in unwanted_patterns {
        name = pattern.gsub(&name, "").trim().to_string();
    }

    for replacement in replacements {
        name = name.replace(replacement.0, replacement.1);
    }

    for pattern in vec![&mut basic_pattern, &mut number_pattern] {
        if let Some(s) = pattern.match_maybe_2(&name) {
            if let Some(num) = parse_f32(s) {
                return num;
            }
        }
    }

    -1.0
}

#[cfg(test)]
mod tests {
    //! Tests for the chapter number parser.
    //! Taken from [Tachiyomi's test suite](https://github.com/tachiyomiorg/tachiyomi/blob/master/domain/src/test/java/tachiyomi/domain/chapter/service/ChapterRecognitionTest.kt).
    fn assert_chapter(title: &str, chapter: &str, expected: f32) {
        assert_eq!(super::parse_chapter_number(title.to_string(), chapter.to_string()), expected);
    }

    #[test]
    fn basic_ch_prefix() {
        let title = "Mokushiroku Alice";

        assert_chapter(title, "Mokushiroku Alice Vol.1 Ch. 4: Misrepresentation", 4.0);
    }

    #[test]
    fn basic_ch_prefix_with_space_after_period() {
        let manga_title = "Mokushiroku Alice";

        assert_chapter(manga_title, "Mokushiroku Alice Vol. 1 Ch. 4: Misrepresentation", 4.0);
    }

    #[test]
    fn basic_ch_prefix_with_decimal() {
        let manga_title = "Mokushiroku Alice";

        assert_chapter(manga_title, "Mokushiroku Alice Vol.1 Ch.4.1: Misrepresentation", 4.1);
        assert_chapter(manga_title, "Mokushiroku Alice Vol.1 Ch.4.4: Misrepresentation", 4.4);
    }

    #[test]
    fn basic_ch_prefix_with_alpha_postfix() {
        let manga_title = "Mokushiroku Alice";

        assert_chapter(manga_title, "Mokushiroku Alice Vol.1 Ch.4.a: Misrepresentation", 4.1);
        assert_chapter(manga_title, "Mokushiroku Alice Vol.1 Ch.4.b: Misrepresentation", 4.2);
        assert_chapter(manga_title, "Mokushiroku Alice Vol.1 Ch.4.extra: Misrepresentation", 4.99);
    }

    #[test]
    fn name_containing_one_number() {
        let manga_title = "Bleach";

        assert_chapter(manga_title, "Bleach 567 Down With Snowwhite", 567.0);
    }

    #[test]
    fn name_containing_one_number_and_decimal() {
        let manga_title = "Bleach";

        assert_chapter(manga_title, "Bleach 567.1 Down With Snowwhite", 567.1);
        assert_chapter(manga_title, "Bleach 567.4 Down With Snowwhite", 567.4);
    }

    #[test]
    fn name_containing_one_number_and_alpha() {
        let manga_title = "Bleach";

        assert_chapter(manga_title, "Bleach 567.a Down With Snowwhite", 567.1);
        assert_chapter(manga_title, "Bleach 567.b Down With Snowwhite", 567.2);
        assert_chapter(manga_title, "Bleach 567.extra Down With Snowwhite", 567.99);
    }

    #[test]
    fn chapter_containing_manga_title_and_number() {
        let manga_title = "Solanin";

        assert_chapter(manga_title, "Solanin 028 Vol. 2", 28.0);
    }

    #[test]
    fn chapter_containing_manga_title_and_number_decimal() {
        let manga_title = "Solanin";

        assert_chapter(manga_title, "Solanin 028.1 Vol. 2", 28.1);
        assert_chapter(manga_title, "Solanin 028.4 Vol. 2", 28.4);
    }

    #[test]
    fn chapter_containing_manga_title_and_number_alpha() {
        let manga_title = "Solanin";

        assert_chapter(manga_title, "Solanin 028.a Vol. 2", 28.1);
        assert_chapter(manga_title, "Solanin 028.b Vol. 2", 28.2);
        assert_chapter(manga_title, "Solanin 028.extra Vol. 2", 28.99);
    }

    #[test]
    fn extreme_case() {
        let manga_title = "Onepunch-Man";

        assert_chapter(manga_title, "Onepunch-Man Punch Ver002 028", 28.0);
    }

    #[test]
    fn extreme_case_with_decimal() {
        let manga_title = "Onepunch-Man";

        assert_chapter(manga_title, "Onepunch-Man Punch Ver002 028.1", 28.1);
        assert_chapter(manga_title, "Onepunch-Man Punch Ver002 028.4", 28.4);
    }

    #[test]
    fn extreme_case_with_alpha() {
        let manga_title = "Onepunch-Man";

        assert_chapter(manga_title, "Onepunch-Man Punch Ver002 028.a", 28.1);
        assert_chapter(manga_title, "Onepunch-Man Punch Ver002 028.b", 28.2);
        assert_chapter(manga_title, "Onepunch-Man Punch Ver002 028.extra", 28.99);
    }

    #[test]
    fn chapter_containing_dot_v2() {
        let manga_title = "random";

        assert_chapter(manga_title, "Vol.1 Ch.5v.2: Alones", 5.0);
    }

    #[test]
    fn number_in_manga_title() {
        let manga_title = "Ayame 14";

        assert_chapter(manga_title, "Ayame 14 1 - The summer of 14", 1.0);
    }

    #[test]
    fn space_between_ch_x() {
        let manga_title = "Mokushiroku Alice";

        assert_chapter(manga_title, "Mokushiroku Alice Vol.1 Ch. 4: Misrepresentation", 4.0);
    }

    #[test]
    fn chapter_title_with_ch_substring() {
        let manga_title = "Ayame 14";

        assert_chapter(manga_title, "Vol.1 Ch.1: March 25 (First Day Cohabiting)", 1.0);
    }

    #[test]
    fn chapter_containing_multiple_zeros() {
        let manga_title = "random";

        assert_chapter(manga_title, "Vol.001 Ch.003: Kaguya Doesn't Know Much", 3.0);
    }

    #[test]
    fn chapter_with_version_before_number() {
        let manga_title = "Onepunch-Man";

        assert_chapter(manga_title, "Onepunch-Man Punch Ver002 086 : Creeping Darkness [3]", 86.0);
    }

    #[test]
    fn version_attached_to_chapter_number() {
        let manga_title = "Ansatsu Kyoushitsu";

        assert_chapter(manga_title, "Ansatsu Kyoushitsu 011v002: Assembly Time", 11.0);
    }

    // Case where the chapter title contains the chapter
    // But wait it's not actual the chapter number.
    #[test]
    fn number_after_manga_title_with_chapter_in_chapter_title_case() {
        let manga_title = "Tokyo ESP";

        assert_chapter(manga_title, "Tokyo ESP 027: Part 002: Chapter 001", 027.0);
    }

    #[test]
    fn unparseable_chapter() {
        let manga_title = "random";

        assert_chapter(manga_title, "Foo", -1.0);
    }

    #[test]
    fn chapter_with_time_in_title() {
        let manga_title = "random";

        assert_chapter(manga_title, "Fairy Tail 404: 00:00", 404.0);
    }

    #[test]
    fn chapter_with_alpha_without_dot() {
        let manga_title = "random";

        assert_chapter(manga_title, "Asu No Yoichi 19a", 19.1);
    }

    #[test]
    fn chapter_title_containing_extra_and_vol() {
        let manga_title = "Fairy Tail";

        assert_chapter(manga_title, "Fairy Tail 404.extravol002", 404.99);
        assert_chapter(manga_title, "Fairy Tail 404 extravol002", 404.99);
    }

    #[test]
    fn chapter_title_containing_omake_and_vol() {
        let manga_title = "Fairy Tail";

        assert_chapter(manga_title, "Fairy Tail 404.omakevol002", 404.98);
        assert_chapter(manga_title, "Fairy Tail 404 omakevol002", 404.98);
    }

    #[test]
    fn chapter_title_containing_special_and_vol() {
        let manga_title = "Fairy Tail";

        assert_chapter(manga_title, "Fairy Tail 404.specialvol002", 404.97);
        assert_chapter(manga_title, "Fairy Tail 404 specialvol002", 404.97);
    }

    #[test]
    fn chapter_title_containing_commas() {
        let manga_title = "One Piece";

        assert_chapter(manga_title, "One Piece 300,a", 300.1);
        assert_chapter(manga_title, "One Piece Ch,123,extra", 123.99);
        assert_chapter(manga_title, "One Piece the sunny, goes swimming 024,005", 24.005);
    }

    #[test]
    fn chapter_title_containing_hyphens() {
        let manga_title = "Solo Leveling";

        assert_chapter(manga_title, "ch 122-a", 122.1);
        assert_chapter(manga_title, "Solo Leveling Ch.123-extra", 123.99);
        assert_chapter(manga_title, "Solo Leveling, 024-005", 24.005);
        assert_chapter(manga_title, "Ch.191-200 Read Online", 191.200);
    }

    #[test]
    fn chapters_containing_season() {
        assert_chapter("D.I.C.E", "D.I.C.E[Season 001] Ep. 007", 7.0);
    }

    #[test]
    fn chapters_in_format_sx_chapter_xx() {
        assert_chapter("The Gamer", "S3 - Chapter 20", 20.0);
    }

    #[test]
    fn chapters_ending_with_s() {
        assert_chapter("One Outs", "One Outs 001", 1.0);
    }

    #[test]
    fn chapters_containing_ordinals() {
        let manga_title = "The Sister of the Woods with a Thousand Young";

        assert_chapter(manga_title, "The 1st Night", 1.0);
        assert_chapter(manga_title, "The 2nd Night", 2.0);
        assert_chapter(manga_title, "The 3rd Night", 3.0);
        assert_chapter(manga_title, "The 4th Night", 4.0);
    }
}
fn main() {
    println!("Run cargo test --example chapter_number instead");
}
