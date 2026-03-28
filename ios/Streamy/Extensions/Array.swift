extension Array {
    func get(_ index: Int) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}
