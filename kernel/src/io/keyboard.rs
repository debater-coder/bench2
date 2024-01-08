use conquer_once::spin::OnceCell;
use conquer_once::TryGetError;
use crossbeam_queue::ArrayQueue;
use pc_keyboard::layouts::Us104Key;
use pc_keyboard::{DecodedKey, EventDecoder, HandleControl, ScancodeSet, ScancodeSet1};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub struct Keyboard {
    scancode_set: ScancodeSet1,
    event_decoder: EventDecoder<Us104Key>,
}

impl Keyboard {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("Keyboard::new should only be called once");

        Keyboard {
            scancode_set: ScancodeSet1::new(),
            event_decoder: EventDecoder::new(Us104Key, HandleControl::Ignore),
        }
    }

    pub(crate) fn push_scancode(scancode: u8) -> Result<(), TryGetError> {
        SCANCODE_QUEUE.try_get()?.push(scancode).unwrap_or_default();

        Ok(())
    }

    pub fn poll_next(&mut self) -> Option<DecodedKey> {
        let next_scancode = SCANCODE_QUEUE.try_get().unwrap().pop();
        if next_scancode.is_none() {
            return None;
        }

        let key_event = self
            .scancode_set
            .advance_state(next_scancode.unwrap())
            .unwrap();

        if key_event.is_none() {
            return None;
        }

        self.event_decoder.process_keyevent(key_event.unwrap())
    }
}
