extern crate futures;
#[macro_use]
extern crate tokio_gql;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate serde_json;

extern crate graphql_parser;
use graphql_parser::query::*;

use std::default::Default;
use tokio_gql::coercion::*;

#[allow(dead_code)]
#[derive(SomethingCompletelyDifferent)]
#[SomethingCompletelyDifferent(path = "tests/basic_schema.graphql")]
struct BasicSchema;

mod star_wars {
    #[allow(dead_code)]
    #[derive(SomethingCompletelyDifferent)]
    #[SomethingCompletelyDifferent(path = "tests/star_wars_schema.graphql")]
    struct ComplexSchema;
}

fn test_coercion<OperationType: CoerceQueryDocument + ::std::fmt::Debug + PartialEq>(
    query: &str,
    expected_result: Result<Vec<OperationType>, CoercionError>,
) {
    let context = tokio_gql::query_validation::ValidationContext::new(serde_json::Map::new());
    test_coercion_with_context(context, query, expected_result);
}

fn test_coercion_with_context<
    OperationType: CoerceQueryDocument + ::std::fmt::Debug + PartialEq,
>(
    context: tokio_gql::query_validation::ValidationContext,
    query: &str,
    expected_result: Result<Vec<OperationType>, CoercionError>,
) {
    let query = parse_query(query).unwrap();
    let fields = OperationType::coerce(&query, &context);

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
    let expected = Ok(vec![Operation::Query {
        selection: vec![
            User::LastName {
                respond: Default::default(),
            },
            User::Greeting {
                respond: Default::default(),
            },
        ],
    }]);
    test_coercion::<Operation>(query, expected);
}

#[test]
fn basic_argument_coercion() {
    let query = r##"
    query {
        sayHello(name: "Emilio")
    }
    "##;
    let expected = Ok(vec![Operation::Query {
        selection: vec![User::SayHello {
            respond: Default::default(),
            name: Some("Emilio".to_string()),
        }],
    }]);
    test_coercion::<Operation>(query, expected);
}

#[test]
fn optional_argument_coercion_none() {
    test_coercion::<Operation>(
        r##"
    query {
        sayHello(name: null)
    }
    "##,
        Ok(vec![Operation::Query {
            selection: vec![User::SayHello {
                respond: Default::default(),
                name: None,
            }],
        }]),
    );
}

#[test]
fn optional_argument_coercion_some() {
    test_coercion::<Operation>(
        r##"
    query {
        sayHello(name: "Pikachu")
    }
    "##,
        Ok(vec![Operation::Query {
            selection: vec![User::SayHello {
                respond: Default::default(),
                name: Some("Pikachu".to_string()),
            }],
        }]),
    );
}

/// We do not consider this as an error because that should be caught at the validation step.
#[test]
fn wrong_argument_name_coercion() {
    test_coercion::<Operation>(
        r##"
    query {
        sayHello(name: 33)
    }
    "##,
        Ok(vec![Operation::Query {
            selection: vec![User::SayHello {
                respond: Default::default(),
                name: None,
            }],
        }]),
    );
}

#[test]
fn wrong_argument_type_coercion() {
    test_coercion::<Operation>(
        r##"
        query {
            sayHello(age: "meow")
        }
        "##,
        Err(CoercionError),
    )
}

#[test]
fn int_argument_coercion() {
    test_coercion::<Operation>(
        r##"
        query {
            double(num: 4)
        }
        "##,
        Ok(vec![Operation::Query {
            selection: vec![User::Double {
                respond: Default::default(),
                num: 4,
            }],
        }]),
    )
}

#[test]
fn multiple_arguments_coercion() {
    test_coercion::<Operation>(
        r###"
        query {
            compare(a: "fourty odd", b: 44)
        }
        "###,
        Ok(vec![Operation::Query {
            selection: vec![User::Compare {
                respond: Default::default(),
                a: Some("fourty odd".to_string()),
                b: Some(44),
            }],
        }]),
    );
}

#[test]
fn coercion_with_optional_variable_on_optional_field() {
    let context = tokio_gql::query_validation::ValidationContext::new(serde_json::Map::new());
    test_coercion_with_context::<Operation>(
        context,
        r###"
        query User($number_string: String) {
            compare(a: $number_string, b: 44)
        }
        "###,
        Ok(vec![Operation::Query {
            selection: vec![User::Compare {
                respond: Default::default(),
                a: None,
                b: Some(44),
            }],
        }]),
    );
}

#[test]
fn required_list_of_required_elements_argument_coercion() {
    test_coercion::<Operation>(
        r###"
        query {
            winningNumbers(numbers: [5, 25, 100])
        }
        "###,
        Ok(vec![Operation::Query {
            selection: vec![User::WinningNumbers {
                respond: Default::default(),
                numbers: vec![5, 25, 100],
            }],
        }]),
    )
}

#[test]
fn optional_list_of_optional_elements_argument_coercion() {
    test_coercion::<Operation>(
        r###"
        query {
            allPrimes(nums: [3, 8, 0, -22])
        }
        "###,
        Ok(vec![Operation::Query {
            selection: vec![User::AllPrimes {
                respond: Default::default(),
                nums: Some(vec![Some(3), Some(8), Some(0), Some(-22)]),
            }],
        }]),
    );
}

#[test]
fn null_argument_coercion() {
    test_coercion::<Operation>(
        r##"
        query {
            sayHello(name: null)
        }
        "##,
        Ok(vec![Operation::Query {
            selection: vec![User::SayHello {
                respond: Default::default(),
                name: None,
            }],
        }]),
    )
}

#[test]
fn required_object_argument_coercion() {
    test_coercion::<Operation>(
        r##"
        query {
            isAGoodDog(dog: {
                name: "Hachi",
                weight: 12,
                has_chip: true,
                vaccinated: true,
            })
        }
        "##,
        Ok(vec![Operation::Query {
            selection: vec![User::IsAGoodDog {
                respond: Default::default(),
                dog: Dog {
                    name: "Hachi".to_string(),
                    weight: 12,
                    vaccinated: Some(true),
                    has_chip: Some(true),
                },
            }],
        }]),
    )
}

#[test]
fn optional_object_argument_coercion_with_null() {
    test_coercion::<Operation>(
        r##"
        query {
            petDog(dog: null)
        }
        "##,
        Ok(vec![Operation::Query {
            selection: vec![User::PetDog {
                respond: Default::default(),
                dog: None,
            }],
        }]),
    )
}

#[test]
fn arguments_with_composed_names() {
    test_coercion::<Operation>(
        r##"
        query {
            petDog(dog: {
                name: "Hachi",
                weight: 12,
                has_chip: false,
            })
        }
        "##,
        Ok(vec![Operation::Query {
            selection: vec![User::PetDog {
                respond: Default::default(),
                dog: Some(Dog {
                    name: "Hachi".to_string(),
                    weight: 12,
                    has_chip: Some(false),
                    vaccinated: None,
                }),
            }],
        }]),
    )
}

#[test]
fn optional_object_argument_coercion_with_value() {
    test_coercion::<Operation>(
        r##"
        query {
            petDog(dog: {
                name: "Hachi",
                weight: 12,
                vaccinated: true,
            })
        }
        "##,
        Ok(vec![Operation::Query {
            selection: vec![User::PetDog {
                respond: Default::default(),
                dog: Some(Dog {
                    name: "Hachi".to_string(),
                    weight: 12,
                    vaccinated: Some(true),
                    has_chip: None,
                }),
            }],
        }]),
    )
}

#[test]
fn field_returning_object() {
    test_coercion::<Operation>(
        r##"
        query {
            getInbox(index: 3) {
                attachments_contain_dog_photos
            }
        }
        "##,
        Ok(vec![Operation::Query {
            selection: vec![User::GetInbox {
                respond: Default::default(),
                selection: vec![Email::AttachmentsContainDogPhotos {
                    respond: Default::default(),
                }],
                index: Some(3),
            }],
        }]),
    )
}

#[test]
fn union_coercion() {
    use self::star_wars;
    test_coercion::<star_wars::Operation>(
        r##"
        query {
            search(text: "Jar Jar Binks") {
                ...on Human {
                    name,
                    homePlanet,
                }
                ...on Droid {
                    name,
                }
            }
        }
        "##,
        Ok(vec![
            star_wars::Operation::Query {
                selection: vec![star_wars::Query::Search {
                    respond: Default::default(),
                    text: Some("Jar Jar Binks".to_string()),
                    selection: vec![
                        star_wars::SearchResult::OnHuman(vec![
                            star_wars::Human::Name {
                                respond: Default::default(),
                            },
                            star_wars::Human::HomePlanet {
                                respond: Default::default(),
                            },
                        ]),
                        star_wars::SearchResult::OnDroid(vec![star_wars::Droid::Name {
                            respond: Default::default(),
                        }]),
                    ],
                }],
            },
            star_wars::Operation::Mutation {
                selection: Vec::new(),
            },
            star_wars::Operation::Subscription {
                selection: Vec::new(),
            },
        ]),
    );
}

#[test]
fn enum_argument_coercion() {
    use self::star_wars;
    test_coercion::<star_wars::Operation>(
        r##"
        query {
            hero(episode: JEDI) {
                name
            }
        }
        "##,
        Ok(vec![
            star_wars::Operation::Query {
                selection: vec![star_wars::Query::Hero {
                    respond: Default::default(),
                    episode: Some(star_wars::Episode::Jedi),
                    selection: vec![star_wars::Character::Name {
                        respond: Default::default(),
                    }],
                }],
            },
            star_wars::Operation::Mutation {
                selection: Vec::new(),
            },
            star_wars::Operation::Subscription {
                selection: Vec::new(),
            },
        ]),
    );
}

#[test]
fn default_values() {
    use self::star_wars;
    test_coercion::<star_wars::Operation>(
        r##"
        query {
            starship(id: "42") {
                length
            }
        }
        "##,
        Ok(vec![
            star_wars::Operation::Query {
                selection: vec![star_wars::Query::Starship {
                    respond: Default::default(),
                    id: "42".to_string(),
                    selection: vec![star_wars::Starship::Length {
                        respond: Default::default(),
                        unit: Some(star_wars::LengthUnit::Meter),
                    }],
                }],
            },
            star_wars::Operation::Mutation {
                selection: Vec::new(),
            },
            star_wars::Operation::Subscription {
                selection: Vec::new(),
            },
        ]),
    )
}

#[test]
fn enum_variable() {
    let variables =
        if let serde_json::Value::Object(map) = json!({ "favourite_episode": { "JEDI": null } }) {
            map
        } else {
            panic!()
        };
    let context = tokio_gql::query_validation::ValidationContext::new(variables);
    test_coercion_with_context::<star_wars::Operation>(
        context,
        r##"
        query Query($favourite_episode: Episode!) {
            hero(episode: $favourite_episode) {
                name
            }
        }
        "##,
        Ok(vec![
            star_wars::Operation::Query {
                selection: vec![star_wars::Query::Hero {
                    respond: Default::default(),
                    episode: Some(star_wars::Episode::Jedi),
                    selection: vec![star_wars::Character::Name {
                        respond: Default::default(),
                    }],
                }],
            },
            star_wars::Operation::Mutation {
                selection: Vec::new(),
            },
            star_wars::Operation::Subscription {
                selection: Vec::new(),
            },
        ]),
    )
}

#[test]
fn string_variable() {
    let variables = if let serde_json::Value::Object(map) =
        json!({ "maybe_millenium_falcon": "Millenium Falcon!!!" })
    {
        map
    } else {
        panic!()
    };
    let context = tokio_gql::query_validation::ValidationContext::new(variables);
    test_coercion_with_context::<star_wars::Operation>(
        context,
        r##"
        query Query($maybe_millenium_falcon: ID) {
            starship(id: $maybe_millenium_falcon) {
                name
            }
        }
        "##,
        Ok(vec![
            star_wars::Operation::Query {
                selection: vec![star_wars::Query::Starship {
                    respond: Default::default(),
                    id: "Millenium Falcon!!!".to_string(),
                    selection: vec![star_wars::Starship::Name {
                        respond: Default::default(),
                    }],
                }],
            },
            star_wars::Operation::Mutation { selection: vec![] },
            star_wars::Operation::Subscription { selection: vec![] },
        ]),
    )
}

#[test]
fn input_object_variable() {
    let variables = if let serde_json::Value::Object(map) =
        json!({ "good_dog": { "name": "Waffles", "weight": 12 } })
    {
        map
    } else {
        panic!()
    };
    let context = tokio_gql::query_validation::ValidationContext::new(variables);
    test_coercion_with_context::<Operation>(
        context,
        r##"
        query User($good_dog: Dog) {
            petDog(dog: $good_dog)
        }
        "##,
        Ok(vec![Operation::Query {
            selection: vec![User::PetDog {
                respond: Default::default(),

                dog: Some(Dog {
                    name: "Waffles".to_string(),
                    weight: 12,
                    has_chip: None,
                    vaccinated: None,
                }),
            }],
        }]),
    )
}

#[test]
fn missing_variables() {
    let variables = if let serde_json::Value::Object(map) = json!({
            "email_index": null,
            "my_number": 43,
        }) {
        map
    } else {
        panic!()
    };
    let context = tokio_gql::query_validation::ValidationContext::new(variables);
    test_coercion_with_context::<Operation>(
        context,
        r##"
        query User($email_index: Int, $my_number: Int!, $verbose_number: String) {
            compare(a: $verbose_number, b: $my_number)
            getInbox(index: $email_index)
            double(num: $my_number)
        }
        "##,
        Ok(vec![Operation::Query {
            selection: vec![
                User::Compare {
                    respond: Default::default(),
                    a: None,
                    b: Some(43),
                },
                User::GetInbox {
                    respond: Default::default(),
                    index: None,
                    selection: vec![],
                },
                User::Double {
                    respond: Default::default(),
                    num: 43,
                },
            ],
        }]),
    )
}

#[test]
fn other_primitive_variable_types() {
    let variables =
        if let serde_json::Value::Object(map) = json!({ "numbers": [12, 83, 38, -20, 10000] }) {
            map
        } else {
            panic!()
        };
    let context = tokio_gql::query_validation::ValidationContext::new(variables);
    test_coercion_with_context::<Operation>(
        context,
        r##"
        query User($numbers: [Int]) {
            allPrimes(nums: $numbers)
        }
        "##,
        Ok(vec![Operation::Query {
            selection: vec![User::AllPrimes {
                respond: Default::default(),
                nums: Some(vec![Some(12), Some(83), Some(38), Some(-20), Some(10000)]),
            }],
        }]),
    );
}

#[test]
fn interface_with_field_and_spread_selection() {
    test_coercion::<star_wars::Operation>(
        r##"
        query {
            character(id: "yoda") {
                id
                name
                appearsIn
                ...on Human {
                    height
                    homePlanet
                }
                ...on Droid {
                    primaryFunction

                }
            }
        }
        "##,
        Ok(vec![
            star_wars::Operation::Query {
                selection: vec![star_wars::Query::Character {
                    respond: Default::default(),
                    id: "yoda".to_string(),
                    selection: vec![
                        star_wars::Character::Id {
                            respond: Default::default(),
                        },
                        star_wars::Character::Name {
                            respond: Default::default(),
                        },
                        star_wars::Character::AppearsIn {
                            respond: Default::default(),
                        },
                        star_wars::Character::OnHuman(vec![
                            star_wars::Human::Height {
                                respond: Default::default(),
                                unit: Some(star_wars::LengthUnit::Meter),
                            },
                            star_wars::Human::HomePlanet {
                                respond: Default::default(),
                            },
                        ]),
                        star_wars::Character::OnDroid(vec![star_wars::Droid::PrimaryFunction {
                            respond: Default::default(),
                        }]),
                    ],
                }],
            },
            star_wars::Operation::Mutation {
                selection: Vec::new(),
            },
            star_wars::Operation::Subscription {
                selection: Vec::new(),
            },
        ]),
    );
}
