const YEAR: u64 = 31_536_000;
const MAX: u64 = YEAR.saturating_mul(100);

#[inline]
fn quotient(value: u64, divisor: u64) -> u64 {
    value.checked_div(divisor).unwrap_or(0)
}

#[inline]
fn remainder(value: u64, divisor: u64) -> u64 {
    value.checked_rem(divisor).unwrap_or(0)
}

#[inline]
fn rounded_divisor(value: u64, divisor: u64) -> u64 {
    quotient(value.saturating_add(quotient(divisor, 2)), divisor)
}

#[inline]
const fn unit_seconds(unit: char) -> Option<u64> {
    match unit {
        'y' => Some(YEAR),
        'd' => Some(86_400),
        'h' => Some(3_600),
        'm' => Some(60),
        's' => Some(1),
        _ => None,
    }
}

#[inline]
const fn next_unit(unit: char) -> Option<char> {
    match unit {
        'y' => Some('d'),
        'd' => Some('h'),
        'h' => Some('m'),
        'm' => Some('s'),
        _ => None,
    }
}

pub fn parse(input: &str) -> Option<u64> {
    let text: String = input
        .to_lowercase()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();

    if text.is_empty() {
        return None;
    }

    let characters: Vec<char> = text.chars().collect();
    let mut index = 0;
    let mut total: u64 = 0;
    let mut last = None;

    while index < characters.len() {
        let start = index;
        while characters.get(index).is_some_and(char::is_ascii_digit) {
            index = index.saturating_add(1);
        }
        if start == index {
            return None;
        }

        let digits: String = characters
            .iter()
            .skip(start)
            .take(index.saturating_sub(start))
            .collect();
        let value: u64 = digits.parse().ok()?;

        let unit = match (characters.get(index), last) {
            (Some(character), _) if unit_seconds(*character).is_some() => {
                index = index.saturating_add(1);
                *character
            }
            (None, None) => 'm',
            (_, Some(previous)) => next_unit(previous)?,
            (Some(_), None) => return None,
        };

        total = total.checked_add(value.checked_mul(unit_seconds(unit)?)?)?;
        if total == 0 || total > MAX {
            return None;
        }
        last = Some(unit);
    }

    Some(total)
}

pub fn split(text: &str) -> Option<(u64, Option<String>)> {
    let input = text.trim();

    if input.is_empty() {
        return None;
    }
    if let Some(seconds) = parse(input) {
        return Some((seconds, None));
    }

    let mut parts = input.splitn(2, char::is_whitespace);
    let token = parts.next()?;
    let seconds = parse(token)?;
    let message = parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);

    Some((seconds, message))
}

pub fn clock(seconds: u64) -> String {
    let days = quotient(seconds, 86_400);
    let hours = quotient(remainder(seconds, 86_400), 3_600);
    let minutes = quotient(remainder(seconds, 3_600), 60);
    let secs = remainder(seconds, 60);

    if days > 0 {
        format!("{days}d {hours}:{minutes:02}:{secs:02}")
    } else if hours > 0 {
        format!("{hours}:{minutes:02}:{secs:02}")
    } else {
        format!("{minutes}:{secs:02}")
    }
}

pub fn human(seconds: u64) -> String {
    let years = quotient(seconds, YEAR);
    let days = quotient(remainder(seconds, YEAR), 86_400);
    let hours = quotient(remainder(seconds, 86_400), 3_600);
    let minutes = quotient(remainder(seconds, 3_600), 60);
    let secs = remainder(seconds, 60);

    let mut parts = Vec::new();
    for (value, unit) in [
        (years, 'y'),
        (days, 'd'),
        (hours, 'h'),
        (minutes, 'm'),
        (secs, 's'),
    ] {
        if value > 0 {
            parts.push(format!("{value}{unit}"));
        }
    }

    if parts.is_empty() {
        "0s".to_owned()
    } else {
        parts.join(" ")
    }
}

pub fn approx(seconds: u64) -> String {
    if seconds < 60 {
        return "1m".to_owned();
    }
    if seconds >= 86_400 {
        let days = rounded_divisor(seconds, 86_400);
        if days >= 365 {
            return format!("{}y", rounded_divisor(days, 365));
        }
        return format!("{days}d");
    }
    if seconds >= 3_600 {
        let hours = rounded_divisor(seconds, 3_600);
        if hours >= 24 {
            return "1d".to_owned();
        }
        return format!("{hours}h");
    }

    let minutes = rounded_divisor(seconds, 60);
    if minutes >= 60 {
        "1h".to_owned()
    } else {
        format!("{minutes}m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_units_in_any_order_returns_seconds() {
        assert_eq!(parse("1h5"), Some(3_900));
        assert_eq!(parse("4s10h3m"), Some(36_184));
        assert_eq!(parse("1y"), Some(YEAR));
    }

    #[test]
    fn test_parse_bare_number_returns_minutes() {
        assert_eq!(parse("5"), Some(300));
    }

    #[test]
    fn test_parse_invalid_input_returns_none() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("abc"), None);
        assert_eq!(parse("1s2"), None);
    }

    #[test]
    fn test_split_duration_and_message_returns_both() {
        let result = split("1h5 wake up").expect("valid input");

        assert_eq!(result, (3_900, Some("wake up".to_owned())));
    }

    #[test]
    fn test_clock_formats_hours_and_minutes() {
        assert_eq!(clock(3_900), "1:05:00");
        assert_eq!(clock(90_061), "1d 1:01:01");
    }

    #[test]
    fn test_human_formats_all_units() {
        assert_eq!(human(0), "0s");
        assert_eq!(human(31_536_000 + 90_061), "1y 1d 1h 1m 1s");
    }

    #[test]
    fn test_approx_rounds_to_nearest_unit() {
        assert_eq!(approx(0), "1m");
        assert_eq!(approx(59), "1m");
        assert_eq!(approx(90), "2m");
        assert_eq!(approx(3_600), "1h");
        assert_eq!(approx(90_000), "1d");
        assert_eq!(approx(400 * 86_400), "1y");
    }

    #[test]
    fn test_helper_functions_cover_edge_cases() {
        assert_eq!(quotient(10, 0), 0);
        assert_eq!(remainder(10, 0), 0);
        assert_eq!(rounded_divisor(10, 3), 3);
        assert_eq!(unit_seconds('x'), None);
        assert_eq!(next_unit('s'), None);
        assert_eq!(split(""), None);
    }
}
