// pub fn read_file(path: String) -> Result<Vec<i16>, hound::Error> {
//     let mut reader = hound::WavReader::open(path).unwrap();
//     let results: Result<Vec<i16>, _> = reader.samples::<i16>().collect();
//     return results;
// }
pub fn buff_to_vec(path: String) -> Vec<i16> {
    let mut reader = hound::WavReader::open(path).unwrap();
    let mut v = vec![0; reader.len().try_into().unwrap()];
    for (j, i) in reader.samples::<i16>().enumerate() {
        v[j] = i.unwrap();
    }
    return v;
}

pub fn convert_vecs<T, U>(v: Vec<T>) -> Vec<U>
where
    T: Into<U>,
{
    v.into_iter().map(Into::into).collect()
}

pub fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
    assert!(!v.is_empty());
    let len = v[0].len();
    let mut iters: Vec<_> = v.into_iter().map(|n| n.into_iter()).collect();
    (0..len)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap())
                .collect::<Vec<T>>()
        })
        .collect()
}
