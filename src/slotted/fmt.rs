use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;

use crate::slotted::Slotted;
use crate::slotted::Pointer;
use crate::slot::SlotBytes;


impl<K, V, P> Debug for Slotted<K, V, P>
    where K: Ord + SlotBytes + Debug,
          V: SlotBytes + Debug,
          P: Pointer + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("Slotted")
            .field("header", &&self.page.bytes[0..8])
            .field("pointers", &self.pointers())
            .field("slots", &self.slots())
            .finish()
    }
}