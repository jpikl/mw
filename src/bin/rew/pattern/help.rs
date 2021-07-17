use indoc::indoc;

pub const PATTERN: &str = indoc! {r#"
# SYNTAX

  `abc`         Constant
  `{}`          Empty expression
  `{x}`         Expression with a filter
  `{x|y|z}`     Expression with multiple filters
  `a{}b{x|y}c`  Mixed constant and expresions.

# RULES

  1. Constants are directly copied to output.
  2. Expression is replaced by input value.
  3. Filters are consecutively applied on input value.

# ESCAPING

  `%/`  System directory separator
  `%n`  Line feed
  `%r`  Carriage return
  `%t`  Horizontal tab
  `%0`  Null
  `%{`  Escaped `{`
  `%|`  Escaped `|`
  `%}`  Escaped `{`
  `%%`  Escaped `%`
"#};

pub const FILTERS: &str = indoc! {r#"
# PATH

  `f`  File name           `d`  Parent directory
  `F`  Last name           `D`  Remove last name

  `b`  Base name           `e`  Extension
  `B`  Remove extension    `E`  Extension with dot

  `w`  Working directory

  `a`  Absolute path       `p`  Normalized path
  `A`  Relative path       `P`  Canonical path

  `z`  Ensure trailing directory separator
  `Z`  Remove trailing directory separator

# SUBSTRING

  `#A-B`  From `A` to `B`         (`A`, `B` = inclusive 1-based index)
  `#A+L`  From `A` of length `L`    (`-A` = backward indexing)
  `#A-`   From `A` to end
  `#A`    Character at `A`

# FIELD

  `&N:S`  Field `N`, string separator `S`   (`:` = any delimiter char except `/`)
  `&N/S`  Field `N`, regex separator `S`    (`N` = 1-based index)
  `&N`    Field `N`, default separator   (`-N` = backward indexing)

# REPLACEMENT

  `r:X:Y`  Replace `X` with `Y`      (`r` = first occurence)
  `r:X`    Remove `X`              (`R` = all occurences)
  `?D`     Replace empty with `D`  (`:` = any delimiter char)

# REGULAR EXPRESSIONS

  `=E`     Match of regex `E`           (`:` = any delimiter char)
  `s:X:Y`  Replace match of `X` with `Y`  (`s` = first occurence)
  `s:X`    Remove match of `X`          (`S` = all occurences)

  `@:X1:Y1:...:Xn:Yn:D`  Output `Yi` for first match of `Xi`, `D` for no match
  `$0`, `$1`, `$2`, ...      Capture group of a global regex or `s/S/@` regex

# FORMATTING

  `t`  Trim
  `^`  To uppercase    `i`   To ASCII
  `v`  To lowercase    `I`   Remove non-ASCII chars

  `*N`    Repeat `N` times
  `<<M`   Left pad with `M`            (`>>` or `>` to right pad)
  `<N:M`  Left pad `N` times with `M`    (`:` = any delimiter char)

# GENERATORS

  `*N:V`  Repeat `N` times `V`    (`:` = any delimiter char)

  `u`  Random 64-bit number    `c`  Local counter
  `U`  Random UUID             `C`  Global counter

  `uA-B`  `u` where `A <= u <= B`
  `uA-`   `u` where `A <= u`
"#};

pub const REGEX_HINT: &str =
    "Visit `https://docs.rs/regex/1/regex/#syntax` for regular expression syntax.";
pub const PATTERN_HINT: &str = "Use `--help-pattern` flag to print pattern syntax reference.";
pub const FILTERS_HINT: &str = "Use `--help-filters` flag to print filter reference.";

#[cfg(test)]
mod tests {
    use common::help::highlight;
    use common::testing::ColoredOuput;
    use ntest::*;
    use test_case::test_case;

    use super::*;

    #[test_case(PATTERN      ; "pattern")]
    #[test_case(FILTERS      ; "filters")]
    #[test_case(PATTERN_HINT ; "pattern hint")]
    #[test_case(FILTERS_HINT ; "filters hint")]
    fn can_highlight(text: &str) {
        let mut output = ColoredOuput::new();
        highlight(&mut output, text).unwrap();
        assert_false!(output.chunks().is_empty());
    }
}
