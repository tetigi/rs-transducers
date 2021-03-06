/*
 * Copyright 2016 rs-transducers developers
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem;

use super::{Transducer, Reducing, StepResult};

pub struct MapTransducer<F> {
    f: F
}

pub struct MapReducer<R, F> {
    rf: R,
    t: MapTransducer<F>
}

impl<F, RI> Transducer<RI> for MapTransducer<F> {
    type RO = MapReducer<RI, F>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        MapReducer {
            rf: reducing_fn,
            t: self
        }
    }
}

impl<R, F, I, O, OF, E> Reducing<I, OF, E> for MapReducer<R, F>
    where F: Fn(I) -> O,
          R: Reducing<O, OF, E> {

    type Item = O;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        self.rf.step((self.t.f)(value))
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn map<F, I, O>(f: F) -> MapTransducer<F>
    where F: Fn(I) -> O {

    MapTransducer {
        f: f
    }
}

pub struct MapIndexedTransducer<F> {
    f: F
}

pub struct MapIndexedReducer<R, F> {
    rf: R,
    t: MapIndexedTransducer<F>,
    count: usize
}

impl<F, RI> Transducer<RI> for MapIndexedTransducer<F> {
    type RO = MapIndexedReducer<RI, F>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        MapIndexedReducer {
            rf: reducing_fn,
            t: self,
            count: 0
        }
    }
}

impl<R, F, I, O, OF, E> Reducing<I, OF, E> for MapIndexedReducer<R, F>
    where F: Fn(usize, I) -> O,
          R: Reducing<O, OF, E> {

    type Item = O;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        let idx = self.count;
        self.count += 1;
        self.rf.step((self.t.f)(idx, value))
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn map_indexed<F, I, O>(f: F) -> MapIndexedTransducer<F>
    where F: Fn(usize, I) -> O {

    MapIndexedTransducer {
        f: f
    }
}

pub struct MapcatTransducer<F> {
    f: F
}

pub struct MapcatReducer<R, F> {
    rf: R,
    t: MapcatTransducer<F>
}

impl<F, RI> Transducer<RI> for MapcatTransducer<F> {
    type RO = MapcatReducer<RI, F>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        MapcatReducer {
            rf: reducing_fn,
            t: self
        }
    }
}

impl<R, F, I, O, IO, OF, E> Reducing<I, OF, E> for MapcatReducer<R, F>
    where IO: IntoIterator<Item=O>,
          F: Fn(I) -> IO,
          R: Reducing<O, OF, E> {

    type Item = O;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        for o in (self.t.f)(value) {
            match self.rf.step(o) {
                Ok(StepResult::Continue) => (),
                Ok(StepResult::Stop) => return Ok(StepResult::Stop),
                Err(e) => return Err(e)
            }
        }
        Ok(StepResult::Continue)
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn mapcat<F, I, O, IO>(f: F) -> MapcatTransducer<F>
    where IO: IntoIterator<Item=O>,
          F: Fn(I) -> IO {

    MapcatTransducer {
        f: f
    }
}

pub struct FilterTransducer<F> {
    f: F,
    inclusive: bool
}

pub struct FilterReducer<R, F> {
    rf: R,
    t: FilterTransducer<F>
}

impl<F, RI> Transducer<RI> for FilterTransducer<F> {
    type RO = FilterReducer<RI, F>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        FilterReducer {
            rf: reducing_fn,
            t: self
        }
    }
}

impl<R, F, I, OF, E> Reducing<I, OF, E> for FilterReducer<R, F>
    where F: Fn(&I) -> bool,
          R: Reducing<I, OF, E> {
    type Item = I;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        let mut include = (self.t.f)(&value);
        if !self.t.inclusive {
            include = !include;
        }
        if include {
            self.rf.step(value)
        } else {
            Ok(StepResult::Continue)
        }
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn filter<F, T>(f: F) -> FilterTransducer<F>
    where F: Fn(&T) -> bool {

    FilterTransducer {
        f: f,
        inclusive: true
    }
}

pub fn remove<F, T>(f: F) -> FilterTransducer<F>
    where F: Fn(&T) -> bool {

    FilterTransducer {
        f: f,
        inclusive: false
    }
}

pub struct KeepTransducer<F>(F);

pub struct KeepReducer<R, F> {
    rf: R,
    t: KeepTransducer<F>
}

impl<F, RI> Transducer<RI> for KeepTransducer<F> {
    type RO = KeepReducer<RI, F>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        KeepReducer {
            rf: reducing_fn,
            t: self
        }
    }
}

impl<R, F, I, O, OF, E> Reducing<I, OF, E> for KeepReducer<R, F>
    where F: Fn(I) -> Option<O>,
          R: Reducing<O, OF, E> {

    type Item = O;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        match (self.t.0)(value) {
            Some(o) => self.rf.step(o),
            None => Ok(StepResult::Continue)
        }
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn keep<F, I, O>(f: F) -> KeepTransducer<F>
    where F: Fn(I) -> Option<O> {

    KeepTransducer(f)
}

pub struct KeepIndexedTransducer<F>(F);

pub struct KeepIndexedReducer<R, F> {
    rf: R,
    t: KeepIndexedTransducer<F>,
    count: usize
}

impl<F, RI> Transducer<RI> for KeepIndexedTransducer<F> {
    type RO = KeepIndexedReducer<RI, F>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        KeepIndexedReducer {
            rf: reducing_fn,
            t: self,
            count: 0
        }
    }
}

impl<R, F, I, O, OF, E> Reducing<I, OF, E> for KeepIndexedReducer<R, F>
    where F: Fn(usize, I) -> Option<O>,
          R: Reducing<O, OF, E> {

    type Item = O;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        let idx = self.count;
        self.count += 1;

        match (self.t.0)(idx, value) {
            Some(o) => self.rf.step(o),
            None => Ok(StepResult::Continue)
        }
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn keep_indexed<F, I, O>(f: F) -> KeepIndexedTransducer<F>
    where F: Fn(usize, I) -> Option<O> {

    KeepIndexedTransducer(f)
}

pub struct PartitionTransducer<T> {
    size: usize,
    all: bool,
    t: PhantomData<T>
}

pub struct PartitionReducer<RF, T> {
    t: PartitionTransducer<T>,
    rf: RF,
    holder: Vec<T>
}

impl<RI, T> Transducer<RI> for PartitionTransducer<T> {
    type RO = PartitionReducer<RI, T>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        let size = self.size;
        PartitionReducer {
            t: self,
            rf: reducing_fn,
            holder: Vec::with_capacity(size)
        }
    }
}

impl<R, I, OF, E> Reducing<I, OF, E> for PartitionReducer<R, I>
    where R: Reducing<Vec<I>, OF, E> {

    type Item = Vec<I>;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        self.holder.push(value);
        if self.holder.len() == self.t.size {
            let mut other_holder = Vec::with_capacity(self.t.size);
            mem::swap(&mut other_holder, &mut self.holder);
            self.rf.step(other_holder)
        } else {
            Ok(StepResult::Continue)
        }
    }

    fn complete(&mut self) -> Result<(), E> {
        if self.t.all {
            let mut other_holder = Vec::new();
            mem::swap(&mut other_holder, &mut self.holder);
            try!(self.rf.step(other_holder));
        }
        self.rf.complete()
    }
}

pub fn partition<T>(num: usize) -> PartitionTransducer<T> {
    PartitionTransducer {
        size: num,
        all: false,
        t: PhantomData
    }
}

pub fn partition_all<T>(num: usize) -> PartitionTransducer<T> {
    PartitionTransducer {
        size: num,
        all: true,
        t: PhantomData
    }
}

pub struct TakeTransducer(usize);

pub struct TakeReducer<RF> {
    rf: RF,
    taken: usize,
    t: TakeTransducer
}

impl<RI> Transducer<RI> for TakeTransducer {
    type RO = TakeReducer<RI>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        TakeReducer {
            rf: reducing_fn,
            taken: 0,
            t: self
        }
    }
}

impl<R, I, OF, E> Reducing<I, OF, E> for TakeReducer<R>
    where R: Reducing<I, OF, E> {

    type Item = I;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        if self.taken < self.t.0 {
            self.taken += 1;
            match self.rf.step(value) {
                Ok(StepResult::Continue) => if self.taken < self.t.0 {
                    Ok(StepResult::Continue)
                } else {
                    Ok(StepResult::Stop)
                },
                Ok(StepResult::Stop) => Ok(StepResult::Stop),
                Err(e) => Err(e)
            }
        } else {
            Ok(StepResult::Stop)
        }
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn take(num: usize) -> TakeTransducer {
    TakeTransducer(num)
}

pub struct TakeWhileTransducer<F>(F);

pub struct TakeWhileReducer<RF, F> {
    rf: RF,
    t: TakeWhileTransducer<F>
}

impl<RI, F> Transducer<RI> for TakeWhileTransducer<F> {
    type RO = TakeWhileReducer<RI, F>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        TakeWhileReducer {
            rf: reducing_fn,
            t: self
        }
    }
}

impl<R, I, OF, E, F> Reducing<I, OF, E> for TakeWhileReducer<R, F>
    where R: Reducing<I, OF, E>,
          F: Fn(&I) -> bool {

    type Item = I;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        if (self.t.0)(&value) {
            self.rf.step(value)
        } else {
            Ok(StepResult::Stop)
        }
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn take_while<F, T>(pred: F) -> TakeWhileTransducer<F>
    where F: Fn(&T) -> bool {

    TakeWhileTransducer(pred)
}

pub struct DropWhileTransducer<F>(F);

pub struct DropWhileReducer<RF, F> {
    rf: RF,
    t: DropWhileTransducer<F>,
    done: bool
}

impl<RI, F> Transducer<RI> for DropWhileTransducer<F> {
    type RO = DropWhileReducer<RI, F>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        DropWhileReducer {
            rf: reducing_fn,
            t: self,
            done: false
        }
    }
}

impl<R, I, OF, E, F> Reducing<I, OF, E> for DropWhileReducer<R, F>
    where R: Reducing<I, OF, E>,
          F: Fn(&I) -> bool {

    type Item = I;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        if self.done {
            self.rf.step(value)
        } else {
            if !(self.t.0)(&value) {
                self.done = true;
                self.rf.step(value)
            } else {
                Ok(StepResult::Continue)
            }
        }
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn drop_while<F, T>(pred: F) -> DropWhileTransducer<F>
    where F: Fn(&T) -> bool {

    DropWhileTransducer(pred)
}

pub struct DropTransducer(usize);

pub struct DropReducer<RF> {
    rf: RF,
    dropped: usize,
    d: DropTransducer
}

impl<RI> Transducer<RI> for DropTransducer {
    type RO = DropReducer<RI>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        DropReducer {
            rf: reducing_fn,
            dropped: 0,
            d: self
        }
    }
}

impl<R, I, OF, E> Reducing<I, OF, E> for DropReducer<R>
    where R: Reducing<I, OF, E> {

    type Item = I;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        if self.dropped < self.d.0 {
            self.dropped += 1;
            Ok(StepResult::Continue)
        } else {
            self.rf.step(value)
        }
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn drop(size: usize) -> DropTransducer {
    DropTransducer(size)
}

pub struct ReplaceTransducer<T>(HashMap<T, T>);

pub struct ReplaceReducer<RF, T> {
    rf: RF,
    t: ReplaceTransducer<T>
}

impl <'a, RI, T> Transducer<RI> for ReplaceTransducer<T> {
    type RO = ReplaceReducer<RI, T>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        ReplaceReducer {
            rf: reducing_fn,
            t: self
        }
    }
}

impl<'a, R, I, OF, E> Reducing<I, OF, E> for ReplaceReducer<R, I>
    where I: Eq + Hash + Clone,
          R: Reducing<I, OF, E> {

    type Item = I;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        let v:I = match self.t.0.get(&value) {
            Some(val) => val.clone(),
            None => value
        };
        self.rf.step(v)
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn replace<T>(replacements: HashMap<T, T>) -> ReplaceTransducer<T> {
    ReplaceTransducer(replacements)
}

pub struct PartitionByTransducer<F, T, R>
    where F: Fn(&T) -> R {

    f: F,
    t: PhantomData<T>
}

pub struct PartitionByReducer<RF, F, T, R>
    where F: Fn(&T) -> R {

    rf: RF,
    t: PartitionByTransducer<F, T, R>,
    holder: Vec<T>,
    last_res: Option<R>
}

impl<RI, F, T, R> Transducer<RI> for PartitionByTransducer<F, T, R>
    where F: Fn(&T) -> R {

    type RO = PartitionByReducer<RI, F, T, R>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        PartitionByReducer {
            rf: reducing_fn,
            t: self,
            holder: Vec::new(),
            last_res: None
        }
    }
}

impl<R, I, OF, E, F, X> Reducing<I, OF, E> for PartitionByReducer<R, F, I, X>
    where R: Reducing<Vec<I>, OF, E>,
          F: Fn(&I) -> X,
          X: Eq {

    type Item = Vec<I>;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        let last_res = self.last_res.take();
        match last_res {
            None => {
                self.last_res = Some((self.t.f)(&value));
                self.holder.push(value);
                Ok(StepResult::Continue)
            },
            Some(ref res) => {
                let new_res = (self.t.f)(&value);
                if res == &new_res {
                    self.holder.push(value);
                    Ok(StepResult::Continue)
                } else {
                    self.last_res = Some(new_res);
                    let mut other_holder = Vec::new();
                    mem::swap(&mut other_holder, &mut self.holder);
                    self.holder.push(value);
                    self.rf.step(other_holder)
                }
            }
        }
    }

    fn complete(&mut self) -> Result<(), E> {
        if self.holder.len() > 0 {
            let mut other_holder = Vec::new();
            mem::swap(&mut other_holder, &mut self.holder);
            try!(self.rf.step(other_holder));
        }
        self.rf.complete()
    }
}

pub fn partition_by<F, T, R>(partition_func: F) -> PartitionByTransducer<F, T, R>
    where F: Fn(&T) -> R {

    PartitionByTransducer {
        f: partition_func,
        t: PhantomData
    }
}

pub struct InterposeTransducer<T>(T);

pub struct InterposeReducer<R, T> {
    first: bool,
    rf: R,
    t: InterposeTransducer<T>
}

impl<RI, T> Transducer<RI> for InterposeTransducer<T> {
    type RO = InterposeReducer<RI, T>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        InterposeReducer {
            first: true,
            rf: reducing_fn,
            t: self
        }
    }
}

impl<R, I, OF, E> Reducing<I, OF, E> for InterposeReducer<R, I>
    where I: Clone,
          R: Reducing<I, OF, E> {

    type Item = I;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        if self.first {
            self.first = false;
        } else {
            match try!(self.rf.step(self.t.0.clone())) {
                StepResult::Continue => (),
                StepResult::Stop => return Ok(StepResult::Stop)
            }
        }
        self.rf.step(value)
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn interpose<T>(separator: T) -> InterposeTransducer<T> {
    InterposeTransducer(separator)
}

pub struct DedupeTransducer<T>(PhantomData<T>);

pub struct DedupeReducer<R, T> {
    last_val: Option<T>,
    rf: R
}

impl<RI, T> Transducer<RI> for DedupeTransducer<T> {
    type RO = DedupeReducer<RI, T>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        DedupeReducer {
            last_val: None,
            rf: reducing_fn
        }
    }
}

impl<R, I, OF, E> Reducing<I, OF, E> for DedupeReducer<R, I>
    where I: Eq + Clone,
          R: Reducing<I, OF, E> {

    type Item = I;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<StepResult, E> {
        if self.last_val.is_none() {
            self.last_val = Some(value.clone());
            self.rf.step(value)
        } else if self.last_val.as_ref().unwrap() == &value {
            Ok(StepResult::Continue)
        } else {
            self.last_val = Some(value.clone());
            self.rf.step(value)
        }
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
    }
}

pub fn dedupe<T>() -> DedupeTransducer<T> {
    DedupeTransducer(PhantomData)
}
