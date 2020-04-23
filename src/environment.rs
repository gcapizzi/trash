use crate::trash;

use anyhow::Error;

pub struct Environment {}

impl Environment {
    pub fn new() -> Environment {
        Environment {}
    }
}

impl trash::Environment for Environment {
    fn var(&self, name: &str) -> Result<String, Error> {
        let value = std::env::var(name)?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::trash::Environment;
    use expect::{
        expect,
        matchers::{equal, result::be_ok},
    };

    #[test]
    fn it_reads_an_environment_variable() {
        let environment = super::Environment::new();

        let cargo_value = environment.var("CARGO");

        expect(&cargo_value).to(be_ok());
        expect(&cargo_value.unwrap()).not_to(equal(""))
    }
}
