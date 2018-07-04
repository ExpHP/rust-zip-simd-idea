This is an attempt to rewrite `faster` with custom Packed types.

### Why a rewrite?

Just because I find it easier to start fresh while learning the design of a library than to go whole hog from the very beginning and integrate it directly into the upstream crate.

Of course, this has some obvious downsides:

* There's no guarantee that any of the stuff I *haven't* yet incorporated will fit into the design.
* It is more difficult to compare the design to the original.

To help address the latter point, here's a laundry list of **big changes:**

* Tuples of vectors implement `Packed`.  Well, okay, they don't yet, but it's trivial to add.
* I introduced `VLists` as variadic tuples of SIMD vectors.  They are basically `frunk`'s HLists but with trait impls like `std::ops::Add` and friends that make them well-suited to our application.  This is used as a fundamental building block for simplifying boilerplatey impls of anything that supports custom zipped types. (technically they are *trees* and not lists since the way I implemented the traits allows the VLists to contain VLists, which might naturally occur in generic user types)

and **smaller changes:**

* Slice arguments are now associated types, and the function signatures in `Packed` have been rendered nigh-unreadable; see the horror that is `PackedGats`. Such is the price we pay...
  * HRRRNK! Slices were bad enough, but in order to support custom vector types, it got worse; There are now also `Ref` and `RefMut` types standing in for what *used* to be `&self` and `&mut self`.  Yeah. I know.
* `scalar_reduce` takes `self` instead of `&self` due to the above
* **Important:** `replace` on std vector types does not modify the original value, so it should not take `&mut self`.
* Widths have type-level integers associated with them so that they can be equated and compared.
* `SimdObject::Scalar` lost it's `Packable` bound (I'm not sure why it was there)

### Troublesome design questions

The toughest question is deciding where to draw the line between which APIs/methods are implemented on:

* **Just individual SIMD vectors:** Currently there's very little that is only implemented on these.
* **The above plus VLists:** Currently there's nothing exclusively implemented for these, I just use them as primitive building blocks for implementing traits on other types. (every impl is eventually at some point backed by an impl on VLists or a single vector)
* **The above plus user-defined types:** This is where I put most functionality currently. For instance, Packed is implemented on custom user types.  Maybe this is too much, as evidenced by the ugly hacks like `PackedGats` that were necessary to make it work.


### So... iterators?

uh... heh... didn't get that far yet. `X_X`

(and of course, that's the greatest design challenge, so without it I'm not sure how much this prototype is worth!)
