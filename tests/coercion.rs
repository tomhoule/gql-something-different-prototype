#[macro_use]
extern crate tokio_gql;
#[macro_use]
extern crate serde_json;

extern crate graphql_parser;
use graphql_parser::query::*;

use tokio_gql::coercion::*;

#[derive(SomethingCompletelyDifferent)]
#[SomethingCompletelyDifferent(path = "tests/basic_schema.graphql")]
struct BasicSchema;

fn test_coercion(query: &str, expected_result: Result<Schema, CoercionError>) {
    let context = tokio_gql::query_validation::ValidationContext::new();
    let query = parse_query(query).unwrap();
    let fields = Schema::coerce(&query, &context);

    assert_eq!(fields, expected_result,)
}

#[test]
fn query_coercion_works() {
    let query = r##"
    query {
        lastName
        greeting
    }
    "##;
    let expected = Ok(Schema {
        query: vec![User::LastName, User::Greeting],
    });
    test_coercion(query, expected);
}

#[test]
fn basic_argument_coercion() {
    let query = r##"
    query {
        sayHello(name: "Emilio")
    }
    "##;
    let expected = Ok(Schema {
        query: vec![
            User::SayHello {
                name: Some("Emilio".to_string()),
            },
        ],
    });
    test_coercion(query, expected);
}

#[test]
fn optional_argument_coercion_none() {
    test_coercion(
        r##"
    query {
        sayHello(name: null)
    }
    "##,
        Ok(Schema {
            query: vec![User::SayHello { name: None }],
        }),
    );
}

#[test]
fn optional_argument_coercion_some() {
    test_coercion(
        r##"
    query {
        sayHello(name: "Pikachu")
    }
    "##,
        Ok(Schema {
            query: vec![
                User::SayHello {
                    name: Some("Pikachu".to_string()),
                },
            ],
        }),
    );
}

/// We do not consider this as an error because that should be caught at the validation step.
#[test]
fn wrong_argument_name_coercion() {
    test_coercion(
        r##"
    query {
        sayHello(name: 33)
    }
    "##,
        Ok(Schema {
            query: vec![User::SayHello { name: None }],
        }),
    );
}

#[test]
fn wrong_argument_type_coercion() {
    let query = r##"
    query {
        sayHello(age: "meow")
    }
    "##;
    let context = tokio_gql::query_validation::ValidationContext::new();
    let query = parse_query(query).unwrap();
    let coerced = Schema::coerce(&query, &context);
    assert_eq!(coerced, Err(CoercionError));
}

#[test]
fn int_argument_coercion() {
    let query = r##"
    query {
        double(num: 4)
    }
    "##;
    let context = tokio_gql::query_validation::ValidationContext::new();
    let query = parse_query(query).unwrap();
    let coerced = Schema::coerce(&query, &context);
    assert_eq!(
        coerced,
        Ok(Schema {
            query: vec![User::Double { num: 4 }],
        })
    )
}

#[test]
fn multiple_arguments_coercion() {
    test_coercion(
        r###"
        query {
            compare(a: "fourty odd", b: 44)
        }
        "###,
        Ok(Schema {
            query: vec![
                User::Compare {
                    a: Some("fourty odd".to_string()),
                    b: Some(44),
                },
            ],
        }),
    );
}

#[test]
fn required_list_of_required_elements_argument_coercion() {
    test_coercion(
        r###"
        query {
            winningNumbers(numbers: [5, 25, 100])
        }
        "###,
        Ok(Schema {
            query: vec![
                User::WinningNumbers {
                    numbers: vec![5, 25, 100],
                },
            ],
        }),
    )
}

#[test]
fn optional_list_of_optional_elements_argument_coercion() {
    test_coercion(
        r###"
        query {
            allPrimes(nums: [3, 8, 0, -22])
        }
        "###,
        Ok(Schema {
            query: vec![
                User::AllPrimes {
                    nums: Some(vec![Some(3), Some(8), Some(0), Some(-22)]),
                },
            ],
        }),
    );
}

#[test]
fn null_argument_coercion() {
    unimplemented!()
}

#[test]
fn optional_object_argument_coercion() {
    unimplemented!()
}

#[test]
fn required_object_argument_coercion() {}
