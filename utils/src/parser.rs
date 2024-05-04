use regex::Regex;

// const DETAILED_CRON_REGEX: &str = r"((?:\d+,)+\d+|(?:(?:\d+|\*)(?:/|-)(?:\d+|\*))|\d+|\*)";

#[derive(Clone)]
struct Constraint {
    pub min: u32,
    pub max: u32,
}

pub fn parse(cron: &str) -> Vec<Vec<u32>> {
    let constraints: Vec<Constraint> = vec![
        Constraint { min: 0, max: 59 }, // Minute
        Constraint { min: 0, max: 23 }, // Hour
        Constraint { min: 1, max: 31 }, // Day of month
        Constraint { min: 1, max: 12 }, // Month
        Constraint { min: 0, max: 7 },  // Day of week
    ];
    // let re = Regex::new(DETAILED_CRON_REGEX).unwrap();

    // let mut string_fields = vec![];
    // for (_, [cron_part]) in re.captures_iter(cron).map(|c| c.extract()) {
    //     string_fields.push(cron_part.to_string());
    // }
    let string_fields: Vec<String> = cron.split(" ").map(|el| el.to_string()).collect();
    let mut fields = vec![];
    if string_fields.len() != 5 {
        todo!("error");
    } else {
        for (i, el) in string_fields.iter().enumerate() {
            fields.push(parse_field(el.to_owned(), constraints[i].clone()));
        }
    }
    fields
}

fn parse_field(value: String, constraint: Constraint) -> Vec<u32> {
    // Replace '*' and '?'
    let re = Regex::new(r"\*|\?").unwrap();
    let result = re.replace_all(&value, &format!("{}-{}", constraint.min, constraint.max));
    let stack = parse_sequence(result.to_string());
    let outranged_numbers: Vec<u32> = stack
        .clone()
        .into_iter()
        .filter(|el| el < &constraint.min || el > &constraint.max)
        .collect();
    if outranged_numbers.is_empty() {
        return stack;
    } else {
        todo!("error")
    }
}

fn parse_sequence(val: String) -> Vec<u32> {
    let mut result: Vec<Vec<u32>> = vec![];
    for seq in val.split(",") {
        result.push(parse_repeat(seq.to_string()));
    }
    return result.concat();
}

fn parse_repeat(val: String) -> Vec<u32> {
    let reps: Vec<&str> = val.split("/").collect();
    match reps.len() {
        2 => {
            return parse_range(
                reps[0].to_string(),
                reps[reps.len() - 1].parse::<u32>().unwrap(),
            )
        }
        1 => return parse_range(val, 1),
        _ => todo!("Error, invalid cron expression"),
    }
}

fn parse_range(val: String, repeat_interval: u32) -> Vec<u32> {
    let range: Vec<&str> = val.split("-").collect();
    match range.len() {
        2 => {
            let min = range[0].parse::<u32>().unwrap();
            let max = range[1].parse::<u32>().unwrap();
            if min > max {
                todo!("error")
            }

            let mut stack: Vec<u32> = vec![];
            let mut repeat_index = repeat_interval.clone();
            for index in min..=max {
                if !stack.contains(&index)
                    && repeat_index > 0
                    && (repeat_index % repeat_interval) == 0
                {
                    repeat_index = 1;
                    stack.push(index);
                } else {
                    repeat_index += 1;
                }
            }
            return stack;
        }
        1 => return vec![val.parse::<u32>().unwrap()],
        _ => todo!("Error, invalid cron expression"),
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{parse, parse_sequence};
    #[test]
    fn simple_sequence() {
        let result = parse_sequence("0,2".to_string());
        assert_eq!(result, vec![0, 2]);
    }
    #[test]
    fn sequence_with_range() {
        let result = parse_sequence("2-20".to_string());
        assert_eq!(
            result,
            vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
        );
    }
    #[test]
    fn sequence_with_repeat_and_range() {
        let result = parse_sequence("0-20/2".to_string());
        assert_eq!(result, vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20]);
    }
    #[test]
    fn cron_with_repeat_and_star() {
        let result = parse("23 0-20/2 * * *");
        assert_eq!(
            result,
            vec![
                vec![23],
                vec![0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20],
                vec![
                    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
                    23, 24, 25, 26, 27, 28, 29, 30, 31
                ],
                vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
                vec![0, 1, 2, 3, 4, 5, 6, 7]
            ]
        );
    }
}
