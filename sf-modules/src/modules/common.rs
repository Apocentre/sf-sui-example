use protos::pb::sui::{CheckpointData, Created};

pub fn get_created_objects(checkpoint_data: &CheckpointData) -> Vec<&Created> {
  let mut created_objects = vec![];

  checkpoint_data.transactions.iter().for_each(|tx| {
    tx.object_changes.iter().for_each(|c| {
      match c.object_change.as_ref().unwrap() {
        protos::pb::sui::object_change::ObjectChange::Created(created) => {
          created_objects.push(created);
        },
        _ => {},
      }
    });
  });

  created_objects
}
