use crate::runtime::{Object, ObjectRef};

pub(super) struct Heap {
    entries: Vec<Option<HeapEntry>>,
    free_slots: Vec<usize>,
}

impl Heap {
    pub(super) fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_slots: Vec::new(),
        }
    }

    pub(super) fn get(&self, reference: ObjectRef) -> &HeapEntry {
        self.entries[reference.0].as_ref().unwrap()
    }

    pub(super) fn get_mut(&mut self, reference: ObjectRef) -> &mut HeapEntry {
        self.entries[reference.0].as_mut().unwrap()
    }

    pub(super) fn sweep(&mut self) {
        for (index, entry) in &mut self.entries.iter_mut().enumerate() {
            match entry {
                Some(value) if !value.reachable => {
                    #[cfg(feature = "gc-debug")]
                    println!("GC FREE   #{} {:?}", index, value.object);
                    entry.take();
                    self.free_slots.push(index);
                }
                Some(value) => {
                    #[cfg(feature = "gc-debug")]
                    println!("KEEP   #{} {:?}", index, value.object);
                    value.reachable = false;
                }
                None => {}
            }
        }
        #[cfg(feature = "gc-debug")]
        println!("{}", "-".repeat(40));
    }

    pub(super) fn allocate(&mut self, object: Object) -> ObjectRef {
        let reference = self.push(object);

        #[cfg(feature = "gc-debug")]
        println!("ALLOC  #{} {:?}", reference.0, self.get(reference).object);

        reference
    }

    pub(super) fn mark(&mut self, reference: ObjectRef) {
        let entry = self.get_mut(reference);

        if entry.reachable {
            return;
        }

        entry.reachable = true;

        #[cfg(feature = "gc-debug")]
        println!("MARK #{}", reference.0);

        let children = self.get(reference).object.references();

        for child in children {
            self.mark(child);
        }
    }

    fn push(&mut self, object: Object) -> ObjectRef {
        let entry = Some(HeapEntry {
            object,
            reachable: false,
        });

        if let Some(index) = self.free_slots.pop() {
            self.entries[index] = entry;
            ObjectRef(index)
        } else {
            let index = self.entries.len();
            self.entries.push(entry);
            ObjectRef(index)
        }
    }
}

pub(super) struct HeapEntry {
    pub(super) object: Object,
    pub(super) reachable: bool,
}
