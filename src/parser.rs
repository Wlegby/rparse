use std::marker::PhantomData;

// ParseResult should be a Result of either (the remaining stuff, the parsed result) or the str
// where it failed
pub type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

// trait parser implements .parse
pub trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;

    // because we already say what should happen when and is called we DO NOT need to call it when
    // implementing Parser
    fn and<P2, Output2>(self, other: P2) -> Pair<Self, P2>
    where
        Self: Sized,
        P2: Parser<'a, Output2>,
    {
        Pair {
            parser1: self,
            parser2: other,
        }
    }

    fn or<P2>(self, other: P2) -> Either<Self, P2>
    where
        Self: Sized,
        P2: Parser<'a, Output>,
    {
        Either {
            parser1: self,
            parser2: other,
        }
    }

    fn map<F, NewOutput>(self, map_fn: F) -> Map<Self, F, Output>
    where
        Self: Sized,
        F: Fn(Output) -> NewOutput,
    {
        Map {
            parser: self,
            map_fn,
            _marker: PhantomData,
        }
    }

    fn opt(self) -> Opt<Self>
    where
        Self: Sized,
    {
        Opt { parser: self }
    }
}

//Implement parser for a random ass function that returns ParseResult
impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(&'a str) -> ParseResult<'a, Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self(input)
    }
}

pub struct Map<P, F, Output> {
    parser: P,
    map_fn: F,
    _marker: PhantomData<Output>,
}

impl<'a, P, F, Output, NewOutput> Parser<'a, NewOutput> for Map<P, F, Output>
where
    P: Parser<'a, Output>,
    F: Fn(Output) -> NewOutput,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, NewOutput> {
        let (next_input, result) = self.parser.parse(input)?;
        Ok((next_input, (self.map_fn)(result)))
    }
}

pub struct Either<P1, P2> {
    parser1: P1,
    parser2: P2,
}

impl<'a, P1, P2, Output> Parser<'a, Output> for Either<P1, P2>
where
    P1: Parser<'a, Output>,
    P2: Parser<'a, Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        let result1 = self.parser1.parse(input);
        match result1 {
            Ok(r) => Ok(r),
            Err(_) => self.parser2.parse(input),
        }
    }
}

pub struct Opt<P> {
    parser: P,
}
impl<'a, P, Output> Parser<'a, Option<Output>> for Opt<P>
where
    P: Parser<'a, Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Option<Output>> {
        match self.parser.parse(input) {
            Ok((next_input, result)) => Ok((next_input, Some(result))),
            // If it fails, we just pretend it succeeded but found nothing.
            Err(_) => Ok((input, None)),
        }
    }
}

// basically lets us chain parsers
pub struct Pair<P1, P2> {
    parser1: P1,
    parser2: P2,
}

impl<'a, P1, P2, R1, R2> Parser<'a, (R1, R2)> for Pair<P1, P2>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    // when we parse we take the output of the first one and bash it into the second one
    fn parse(&self, input: &'a str) -> ParseResult<'a, (R1, R2)> {
        let (next, result1) = self.parser1.parse(input)?;
        let (remainder, result2) = self.parser2.parse(next)?;
        Ok((remainder, (result1, result2)))
    }
}

// yeah just basically returns and removes a expected string
pub fn literal<'a>(expected: &'a str) -> impl Parser<'a, ()> {
    move |input: &'a str| match input.strip_prefix(expected) {
        Some(rest) => Ok((rest, ())),
        None => Err(input),
    }
}

pub fn split_until<'a>(pattern: &'a str) -> impl Parser<'a, &'a str> {
    move |input: &'a str| {
        let byte_id: usize;

        match input.find(pattern) {
            Some(idx) => byte_id = idx,
            None => return Err(input),
        }

        Ok((&input[byte_id..], &input[..byte_id]))
    }
}

pub fn split_at<'a>(pattern: &'a str) -> impl Parser<'a, &'a str> {
    move |input: &'a str| {
        let byte_id: usize;

        match input.find(pattern) {
            Some(idx) => byte_id = idx,
            None => return Err(input),
        }

        Ok((&input[byte_id + pattern.len()..], &input[..byte_id]))
    }
}

// checks removes and returns if the first char matches the predicate (which is a function)
pub fn match_char<'a, F>(predicate: F) -> impl Parser<'a, char>
where
    F: Fn(char) -> bool,
{
    move |input: &'a str| {
        let mut chars = input.chars();
        match chars.next() {
            Some(c) if predicate(c) => Ok((chars.as_str(), c)),
            _ => Err(input),
        }
    }
}

// returns the string until it reaches a char that matches the predicate
pub fn take_until_inclusive<'a, F>(predicate: F) -> impl Parser<'a, &'a str>
where
    F: Fn(char) -> bool,
{
    move |input: &'a str| {
        let mut bytes_len = 0;
        for c in input.chars() {
            if predicate(c) {
                bytes_len += c.len_utf8();
                break;
            } else {
                bytes_len += c.len_utf8();
            }
        }
        if bytes_len == 0 {
            Err(input)
        } else {
            Ok((&input[bytes_len..], &input[..bytes_len]))
        }
    }
}
pub fn take_until<'a, F>(predicate: F) -> impl Parser<'a, &'a str>
where
    F: Fn(char) -> bool,
{
    move |input: &'a str| {
        let mut bytes_len = 0;
        for c in input.chars() {
            if predicate(c) {
                break;
            } else {
                bytes_len += c.len_utf8();
            }
        }
        if bytes_len == 0 {
            Err(input)
        } else {
            Ok((&input[bytes_len..], &input[..bytes_len]))
        }
    }
}

// returns the string while the character meets the predicate
pub fn take_while<'a, F>(predicate: F) -> impl Parser<'a, &'a str>
where
    F: Fn(char) -> bool,
{
    move |input: &'a str| {
        let mut bytes_len = 0;
        for c in input.chars() {
            if predicate(c) {
                bytes_len += c.len_utf8();
            } else {
                break;
            }
        }
        if bytes_len == 0 {
            Err(input)
        } else {
            Ok((&input[bytes_len..], &input[..bytes_len]))
        }
    }
}

// counts the number of characters in a row that met the predicate
pub fn count_while<'a, F>(predicate: F) -> impl Parser<'a, usize>
where
    F: Fn(char) -> bool,
{
    move |input: &'a str| {
        let mut count = 0;
        let mut bytes_len = 0;

        for c in input.chars() {
            if predicate(c) {
                count += 1;
                bytes_len += c.len_utf8();
            } else {
                break;
            }
        }

        if count == 0 {
            Err(input)
        } else {
            Ok((&input[bytes_len..], count))
        }
    }
}

// returns the string inside the two delimiters
pub fn delimited<'a>(opening: &'a str, closing: &'a str) -> impl Parser<'a, &'a str> {
    move |input: &'a str| {
        let after_bound = match input.strip_prefix(opening) {
            Some(rest) => rest,
            None => return Err(input),
        };

        match after_bound.find(closing) {
            Some(idx) => {
                let inside = &after_bound[..idx];
                let remaining = &after_bound[idx + closing.len()..];
                return Ok((remaining, inside));
            }
            None => Err(input),
        }
    }
}
