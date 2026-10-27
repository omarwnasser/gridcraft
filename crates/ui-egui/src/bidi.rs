//! Arabic script detection, right-to-left alignment helpers, and Presentation Forms-B shaping.
//!
//! With `epaint` 0.37 (`harfrust` + `unicode-bidi`), `egui` natively reorders right-to-left runs
//! visually while keeping each glyph's logical character index identical to the source string.
//! Preserving logical order in [`shape_arabic`] ensures that clicking the first word of an Arabic
//! sentence (drawn on the right) maps to the first word's character indices rather than the end
//! of the sentence.

use egui::Align;

/// Returns `true` when `c` belongs to an Arabic Unicode block.
pub fn is_arabic(c: char) -> bool {
    matches!(c, '\u{0600}'..='\u{06FF}' | '\u{0750}'..='\u{077F}' | '\u{08A0}'..='\u{08FF}' | '\u{FB50}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFC}')
}

/// Returns `true` when `text` contains at least one Arabic character.
pub fn has_arabic(text: &str) -> bool {
    text.chars().any(is_arabic)
}

/// Returns `Align::RIGHT` for text containing Arabic, `Align::LEFT` otherwise.
pub fn text_align(text: &str) -> Align {
    if has_arabic(text) { Align::RIGHT } else { Align::LEFT }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JoiningType {
    NonJoining,
    RightJoining,
    DualJoining,
    Transparent,
}

fn joining_type(c: char) -> JoiningType {
    let u = c as u32;
    if (0x064B..=0x065F).contains(&u) || u == 0x0670 {
        return JoiningType::Transparent;
    }
    match u {
        0x0622 | 0x0623 | 0x0624 | 0x0625 | 0x0627 | 0x0629 | 0x062F | 0x0630 | 0x0631 | 0x0632 | 0x0648 | 0x0649 | 0x0671 => {
            JoiningType::RightJoining
        }
        0x0626 | 0x0628 | 0x062A | 0x062B | 0x062C | 0x062D | 0x062E | 0x0633 | 0x0634 | 0x0635 | 0x0636 | 0x0637 | 0x0638 | 0x0639 | 0x063A
        | 0x0640 | 0x0641 | 0x0642 | 0x0643 | 0x0644 | 0x0645 | 0x0646 | 0x0647 | 0x064A => JoiningType::DualJoining,
        _ => JoiningType::NonJoining,
    }
}

/// (isolated, final, initial, medial) glyph mappings in Presentation Forms-B.
fn glyph_forms(c: char) -> Option<(char, char, char, char)> {
    let (iso, fin, init, med) = match c as u32 {
        0x0621 => (0xFE80, 0xFE80, 0xFE80, 0xFE80), // Hamza
        0x0622 => (0xFE81, 0xFE82, 0xFE81, 0xFE82), // Alef Madda
        0x0623 => (0xFE83, 0xFE84, 0xFE83, 0xFE84), // Alef Hamza Above
        0x0624 => (0xFE85, 0xFE86, 0xFE85, 0xFE86), // Waw Hamza
        0x0625 => (0xFE87, 0xFE88, 0xFE87, 0xFE88), // Alef Hamza Below
        0x0626 => (0xFE89, 0xFE8A, 0xFE8B, 0xFE8C), // Yeh Hamza
        0x0627 => (0xFE8D, 0xFE8E, 0xFE8D, 0xFE8E), // Alef
        0x0628 => (0xFE8F, 0xFE90, 0xFE91, 0xFE92), // Beh
        0x0629 => (0xFE93, 0xFE94, 0xFE93, 0xFE94), // Teh Marbuta
        0x062A => (0xFE95, 0xFE96, 0xFE97, 0xFE98), // Teh
        0x062B => (0xFE99, 0xFE9A, 0xFE9B, 0xFE9C), // Theh
        0x062C => (0xFE9D, 0xFE9E, 0xFE9F, 0xFEA0), // Jeem
        0x062D => (0xFEA1, 0xFEA2, 0xFEA3, 0xFEA4), // Hah
        0x062E => (0xFEA5, 0xFEA6, 0xFEA7, 0xFEA8), // Khah
        0x062F => (0xFEA9, 0xFEAA, 0xFEA9, 0xFEAA), // Dal
        0x0630 => (0xFEAB, 0xFEAC, 0xFEAB, 0xFEAC), // Thal
        0x0631 => (0xFEAD, 0xFEAE, 0xFEAD, 0xFEAE), // Reh
        0x0632 => (0xFEAF, 0xFEB0, 0xFEAF, 0xFEB0), // Zain
        0x0633 => (0xFEB1, 0xFEB2, 0xFEB3, 0xFEB4), // Seen
        0x0634 => (0xFEB5, 0xFEB6, 0xFEB7, 0xFEB8), // Sheen
        0x0635 => (0xFEB9, 0xFEBA, 0xFEBB, 0xFEBC), // Sad
        0x0636 => (0xFEBD, 0xFEBE, 0xFEBF, 0xFEC0), // Dad
        0x0637 => (0xFEC1, 0xFEC2, 0xFEC3, 0xFEC4), // Tah
        0x0638 => (0xFEC5, 0xFEC6, 0xFEC7, 0xFEC8), // Zah
        0x0639 => (0xFEC9, 0xFECA, 0xFECB, 0xFECC), // Ain
        0x063A => (0xFECD, 0xFECE, 0xFECF, 0xFED0), // Ghain
        0x0640 => (0x0640, 0x0640, 0x0640, 0x0640), // Tatweel
        0x0641 => (0xFED1, 0xFED2, 0xFED3, 0xFED4), // Feh
        0x0642 => (0xFED5, 0xFED6, 0xFED7, 0xFED8), // Qaf
        0x0643 => (0xFED9, 0xFEDA, 0xFEDB, 0xFEDC), // Kaf
        0x0644 => (0xFEDD, 0xFEDE, 0xFEDF, 0xFEE0), // Lam
        0x0645 => (0xFEE1, 0xFEE2, 0xFEE3, 0xFEE4), // Meem
        0x0646 => (0xFEE5, 0xFEE6, 0xFEE7, 0xFEE8), // Noon
        0x0647 => (0xFEE9, 0xFEEA, 0xFEEB, 0xFEEC), // Heh
        0x0648 => (0xFEED, 0xFEEE, 0xFEED, 0xFEEE), // Waw
        0x0649 => (0xFEEF, 0xFEF0, 0xFEEF, 0xFEF0), // Alef Maksura
        0x064A => (0xFEF1, 0xFEF2, 0xFEF3, 0xFEF4), // Yeh
        _ => return None,
    };
    Some((char::from_u32(iso).unwrap_or(c), char::from_u32(fin).unwrap_or(c), char::from_u32(init).unwrap_or(c), char::from_u32(med).unwrap_or(c)))
}

fn lam_alef(alef: char) -> Option<(char, char)> {
    let (iso, fin) = match alef as u32 {
        0x0622 => (0xFEF5, 0xFEF6),
        0x0623 => (0xFEF7, 0xFEF8),
        0x0625 => (0xFEF9, 0xFEFA),
        0x0627 => (0xFEFB, 0xFEFC),
        _ => return None,
    };
    Some((char::from_u32(iso).unwrap_or(alef), char::from_u32(fin).unwrap_or(alef)))
}

/// Shapes an Arabic character sequence into Presentation Forms-B glyphs with cursive joining.
pub fn shape_characters(chars: &[char]) -> Vec<char> {
    let mut out = Vec::with_capacity(chars.len());
    let mut i = 0;
    let n = chars.len();
    while let Some(&c) = chars.get(i) {
        if joining_type(c) == JoiningType::Transparent {
            out.push(c);
            i += 1;
            continue;
        }

        // Check if previous character joins forward.
        let mut prev_joins = false;
        let mut j = i;
        while j > 0 {
            j -= 1;
            if let Some(&prev) = chars.get(j)
                && joining_type(prev) != JoiningType::Transparent
            {
                if joining_type(prev) == JoiningType::DualJoining {
                    prev_joins = true;
                }
                break;
            }
        }

        // Check Lam-Alef ligature.
        if c == '\u{0644}' {
            let mut k = i + 1;
            while k < n && chars.get(k).is_some_and(|&ch| joining_type(ch) == JoiningType::Transparent) {
                k += 1;
            }
            if let Some(&next_ch) = chars.get(k)
                && let Some((iso, fin)) = lam_alef(next_ch)
            {
                out.push(if prev_joins { fin } else { iso });
                i = k + 1;
                continue;
            }
        }

        // Check if next character joins backward.
        let mut next_joins = false;
        let mut k = i + 1;
        while let Some(&next_ch) = chars.get(k) {
            let jt = joining_type(next_ch);
            if jt != JoiningType::Transparent {
                if jt == JoiningType::DualJoining || jt == JoiningType::RightJoining {
                    next_joins = true;
                }
                break;
            }
            k += 1;
        }

        if let Some((iso, fin, init, med)) = glyph_forms(c) {
            match joining_type(c) {
                JoiningType::DualJoining => {
                    let g = if prev_joins && next_joins {
                        med
                    } else if prev_joins {
                        fin
                    } else if next_joins {
                        init
                    } else {
                        iso
                    };
                    out.push(g);
                }
                JoiningType::RightJoining => {
                    let g = if prev_joins { fin } else { iso };
                    out.push(g);
                }
                _ => out.push(iso),
            }
        } else {
            out.push(c);
        }
        i += 1;
    }
    out
}

/// Returns `text` in its logical character order for `epaint` 0.37's native `unicode-bidi` and
/// `harfrust` pipeline. Keeping logical character indices intact guarantees that cursor hit-testing
/// (`Galley::cursor_from_pos`) maps clicks on the first word of an Arabic sentence (visually on
/// the right) to character indices `0..first_word_len` instead of the end of the sentence.
pub fn shape_arabic(text: &str) -> String {
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes_arabic_characters_with_ligatures() {
        assert!(!has_arabic("Hello world"));
        assert!(has_arabic("مرحبا"));

        let shaped: String = shape_characters(&"ملف".chars().collect::<Vec<_>>()).into_iter().collect();
        assert_ne!(shaped, "ملف");
        assert!(shaped.chars().any(|c| (0xFE70..=0xFEFC).contains(&(c as u32))));

        let shaped_art: String = shape_characters(&"الرسم".chars().collect::<Vec<_>>()).into_iter().collect();
        assert!(shaped_art.chars().any(|c| (0xFE70..=0xFEFC).contains(&(c as u32))));
    }

    #[test]
    fn preserves_logical_order_so_first_word_indices_match_editor_text() {
        let text = "مرحبا بكم في التطبيق";
        let displayed = shape_arabic(text);
        assert_eq!(displayed, text);
        // The first word "مرحبا" stays at logical character indices 0..5 so clicking it edits the
        // first word, never the end of the sentence ("التطبيق").
        let first_word: String = displayed.chars().take(5).collect();
        assert_eq!(first_word, "مرحبا");
    }

    #[test]
    fn preserves_latin_runs_and_numbers_in_mixed_text() {
        let text = "الاسم: Omar 42";
        let displayed = shape_arabic(text);
        assert!(displayed.contains("Omar"));
        assert!(displayed.contains("42"));
        assert_eq!(displayed, text);
    }

    #[test]
    fn leaves_latin_unmodified() {
        assert_eq!(shape_arabic("Sheet1!A1"), "Sheet1!A1");
        assert_eq!(shape_arabic("12345"), "12345");
        assert_eq!(shape_arabic("=SUM(A1:B10)"), "=SUM(A1:B10)");
    }

    #[test]
    fn arabic_text_uses_right_alignment() {
        assert_eq!(text_align("مرحبا"), Align::RIGHT);
        assert_eq!(text_align("Hello"), Align::LEFT);
    }
}
