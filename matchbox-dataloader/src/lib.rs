extern crate crossbeam_deque;
extern crate futures;

use crossbeam_deque::{Deque, Steal};
use futures::channel::oneshot;
use futures::prelude::*;
use std::collections::HashMap;

/// There should be only one impl of LoadMore, for zero or more impls of DataLoader.
pub trait LoadMore {
    fn load_more(&self) -> Box<Future<Item = (), Error = ()>>;
}

/// The DataLoader always accepts more attached nodes. When called with `load_more`, it should try to resolve them all.
pub trait DataLoader<Item>: LoadMore
where
    Item: HasId,
{
    type Error;

    fn attach(&self, ids: &[<Item as HasId>::Id]) -> oneshot::Receiver<Vec<Option<Item>>>;
}

pub struct DataLoaderContainer<Id, Item> {
    attached: Deque<(oneshot::Sender<Vec<Option<Item>>>, Vec<Id>)>,
}

pub trait HasId {
    type Id;

    fn id(&self) -> Self::Id;
}

impl<Id: Eq + ::std::hash::Hash + Clone, Item: Clone + HasId<Id = Id>>
    DataLoaderContainer<Id, Item>
{
    pub fn new() -> Self {
        DataLoaderContainer {
            attached: Deque::new(),
        }
    }

    pub fn attach(&self, ids: &[Id]) -> oneshot::Receiver<Vec<Option<Item>>> {
        let (sender, receiver) = oneshot::channel();
        self.attached.push((sender, ids.to_vec()));
        receiver
    }

    pub fn get_ids(&self) -> Vec<Id> {
        let mut result = Vec::new();
        let mut buffer = Vec::new();
        loop {
            if self.attached.is_empty() {
                break;
            } else {
                if let Steal::Data((sender, ids)) = self.attached.steal() {
                    result.extend_from_slice(&ids);
                    buffer.push((sender, ids));
                }
            }
        }

        for elem in buffer.into_iter() {
            self.attached.push(elem);
        }

        result
    }

    pub fn resolve(&self, items: impl Iterator<Item = Item>) -> Result<(), ()> {
        let map: HashMap<Id, Item> = items.map(|item| (item.id(), item)).collect();
        while let Some((sender, ids)) = self.attached.pop() {
            let mut payload: Vec<Option<Item>> = ids.iter()
                .map(|id| map.get(id).map(|i| i.clone()))
                .collect();
            sender.send(payload).map_err(|_| ())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct User {
        id: i32,
        name: String,
    }

    impl HasId for User {
        type Id = i32;

        fn id(&self) -> i32 {
            self.id
        }
    }

    struct RequestContext {
        loader: DataLoaderContainer<i32, User>,
    }

    impl DataLoader<User> for RequestContext {
        type Error = ();

        fn load_more(&self) -> Box<Future<Item = (), Error = ()>> {
            // panic!("loading more");
            let result: Vec<User> = self.loader
                .get_ids()
                .iter()
                .map(|id| User {
                    id: *id,
                    name: format!("Mr or Mrs. {}", id),
                })
                .collect();
            self.loader.resolve(result.into_iter());
            Box::new(futures::future::ok(()))
        }

        fn attach(&self, ids: &[i32]) -> oneshot::Receiver<Vec<Option<User>>> {
            self.loader.attach(ids)
        }
    }

    #[test]
    fn simple_data_loader_test() {
        let context = RequestContext {
            loader: DataLoaderContainer::new(),
        };
        let receiver_1 = context.attach(&[33, 18, 22]);
        let receiver_2 = context.attach(&[1, 18, 11]);
        let fut = context
            .load_more()
            .and_then(|_| receiver_1.join(receiver_2).map_err(|_| ()));
        let (result_1, result_2) = futures::executor::block_on(fut).unwrap();
        assert_eq!(
            result_1,
            vec![
                Some(User {
                    id: 33,
                    name: "Mr or Mrs. 33".to_string(),
                }),
                Some(User {
                    id: 18,
                    name: "Mr or Mrs. 18".to_string(),
                }),
                Some(User {
                    id: 22,
                    name: "Mr or Mrs. 22".to_string(),
                }),
            ]
        );
        assert_eq!(
            result_2,
            vec![
                Some(User {
                    id: 1,
                    name: "Mr or Mrs. 1".to_string(),
                }),
                Some(User {
                    id: 18,
                    name: "Mr or Mrs. 18".to_string(),
                }),
                Some(User {
                    id: 11,
                    name: "Mr or Mrs. 11".to_string(),
                }),
            ]
        );
    }
}
