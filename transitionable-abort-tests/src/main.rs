fn main() {
    transitionable::tests::transition_works();
    transitionable::tests::is_poisoned_works();
    transitionable::tests::deref_works();
    transitionable::tests::deref_mut_works();
}
