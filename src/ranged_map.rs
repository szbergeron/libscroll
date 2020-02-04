pub trait ToKey<KeyType> {
    fn to_key(&self) -> KeyType;
}

pub struct RangedMap<Key, Val> where Key: Ord + Copy, Val: ToKey<Key> + Copy {
    //stride: Key, // use for later if it becomes more efficient to do a binary search than a
    //linear one for the size of data used
    map: std::collections::BTreeMap<Key, Val>,
    //default: 
}

impl<Key, Val> RangedMap<Key, Val> where Key: Ord + Copy, Val: ToKey<Key> + Copy {
    pub fn new() -> RangedMap<Key, Val> {
        RangedMap { map: std::collections::BTreeMap::new() }
    }

    pub fn get_neighbors_to(&self, point: Key) -> (Val, Val) {
        //panic!("not impl");
        // find first point
        let p1 = self.iter().rev().find(|&(&k, &_)| { k <= point }).expect("Couldn't find any preceding or equal point");
        let p2 = self.iter()      .find(|&(&k, &_)| { k > point }).expect("Couldn't find following point");

        (*p1.1, *p2.1)
    }

    /// Allows taking a point that we want to get the next, previous, and next-next (if same as a
    /// point already here)
    pub fn get_2nd_neighbors_to(&self, point: Key) -> (Val, Val, Val, Val) {
        let (inner_1, inner_2) = self.get_neighbors_to(point);
        let outer_1 = self.iter().rev().find(|&(&k, &_)| { k < inner_1.to_key() }).expect("Couldn't find any preceding point").1;
        let outer_2 = self.iter()      .find(|&(&k, &_)| { k > inner_2.to_key() }).expect("Couldn't find any following point").1;

        (*outer_1, inner_1, inner_2, *outer_2)
    }

    /*pub fn get_before(&self, point: Key) -> Option<Val> {
        let v = self.iter().rev().find(|&(&k, &_)| { k < point });

        v.map(|(&_, &v)| v ).or_else(|| {  // get value if exists as Some(v) else None
    }

    pub fn get_after(&self, point: Key) -> Option<Val> {
        let v = self.iter().find(|&(&k, &_)| { k > point });

        v.map(|(&_, &v)| v )
    }

    pub fn get_all_after(&self, point: Key) -> Vec<(Key, Val)> {
        self.iter().filter(|&(&k, &_)| { k > point }).map(|(&k, &v)| (k, v)).collect()
    }

    pub fn insert(&mut self, obj: Val) {
        self.map.insert(obj.to_key(), obj);
    }*/

    /*pub fn insert(&mut self, k: Key, v: Val) {
        self.map.insert(k, v);
    }*/
}

impl<Key, Val> std::ops::Deref for RangedMap<Key, Val> where Key: Ord + Copy, Val: ToKey<Key> + Copy {
    type Target = std::collections::BTreeMap<Key, Val>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<Key, Val> std::ops::DerefMut for RangedMap<Key, Val> where Key: Ord + Copy, Val: ToKey<Key> + Copy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
