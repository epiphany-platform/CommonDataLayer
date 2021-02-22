use std::{cmp::max, collections::LinkedList, sync::Mutex};

use log::trace;
use rdkafka::{
    consumer::{DefaultConsumerContext, StreamConsumer},
    message::BorrowedMessage,
    Message, Offset, TopicPartitionList,
};
use vec_map::VecMap;

use crate::abort_on_poison;

#[derive(Default)]
pub struct KafkaAckQueue {
    queue: Mutex<VecMap<KafkaPartitionAckQueue>>,
}
impl KafkaAckQueue {
    pub fn add(&self, message: &BorrowedMessage) {
        let partition = message.partition() as usize;
        let mut queue = self.queue.lock().unwrap_or_else(abort_on_poison);
        let partition_queue = (*queue).entry(partition).or_insert_with(|| {
            KafkaPartitionAckQueue::new(message.topic().to_owned(), message.partition().to_owned())
        });

        partition_queue.add(message);
    }
    pub fn ack(
        &self,
        message: &BorrowedMessage<'_>,
        consumer: &StreamConsumer<DefaultConsumerContext>,
    ) {
        let partition = message.partition() as usize;
        let mut queue = self.queue.lock().unwrap_or_else(abort_on_poison);
        let partition_queue = (*queue).get_mut(partition).unwrap();
        partition_queue.ack(message, consumer);
    }
}

#[derive(Debug)]
pub struct KafkaPartitionAckQueue {
    waiting_list: LinkedList<i64>,
    topic: String,
    partition: i32,
    last_offset: i64,
}

impl KafkaPartitionAckQueue {
    pub fn new(topic: String, partition: i32) -> KafkaPartitionAckQueue {
        KafkaPartitionAckQueue {
            partition,
            topic,
            waiting_list: LinkedList::new(),
            last_offset: 0,
        }
    }

    pub fn add(&mut self, message: &BorrowedMessage<'_>) {
        let offset = message.offset();
        self.waiting_list.push_back(offset);
        self.last_offset = offset;
        trace!("Adding offset {} Partition: {}", offset, self.partition);
    }

    pub fn ack(
        &mut self,
        message: &BorrowedMessage<'_>,
        consumer: &StreamConsumer<DefaultConsumerContext>,
    ) {
        let mut offset = message.offset();
        if *self.waiting_list.front().unwrap() == offset {
            self.waiting_list.pop_front();
            offset = if self.waiting_list.is_empty() {
                max(self.last_offset, offset)
            } else {
                self.waiting_list.front().unwrap() - 1
            };
            trace!(
                "Storing offset {} acknowledged msg {}  List: {:?}",
                offset,
                message.offset(),
                self.waiting_list
            );
            let mut partition_offsets = TopicPartitionList::new();
            partition_offsets.add_partition_offset(
                &self.topic,
                self.partition,
                Offset::Offset(offset),
            );
            rdkafka::consumer::Consumer::store_offsets(consumer, &partition_offsets).unwrap();
        } else {
            trace!("Marking offset {} as processed", offset);
            let mut item = self.waiting_list.cursor_front_mut();
            loop {
                item.move_next();
                if item.current() == Some(&mut offset) {
                    item.remove_current();
                    break;
                }
            }
        }
    }
}
