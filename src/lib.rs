mod parser;
pub use parser::*;

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn repeater_test() {
        let a = count_while(|c| c == '#').parse("##### aösdlj");
        match a {
            Ok((rest, heading_depth)) => {
                println!(
                    "Repeated '#' {} times, followed by: \"{}\"",
                    heading_depth, rest
                );
            }
            Err(i) => println!("Error at: {}", i),
        }
    }
    #[test]
    fn delimiter_test() {
        let a = delimited("**", "**").parse("**This is a bold text** this is non bold text");
        match a {
            Ok((rest, inside)) => {
                println!(
                    "Found Bold text: \"{}\" folowed by non bold text: \"{}\"",
                    inside, rest
                );
            }
            Err(i) => println!("Error at: {}", i),
        }
    }
}
