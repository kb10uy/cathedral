use lyricism::{query_delete, query_insert, query_replace, query_substring, Lyricism};

fn main() {
    let jojo = Lyricism::new(query_insert, query_delete, query_replace, query_substring);

    let test_distances = |q, t| {
        let d = jojo.distance(q, t);
        println!("Query '{q}' / Target '{t}' => {d}");
    };

    test_distances("A", "A");
    test_distances("A", "AA");
    test_distances("AA", "A");
    test_distances("AA", "AA");
    test_distances("zenith", "Zenith");
    test_distances("zenith", "ZEИITH");
    test_distances("ZENITH", "Zenith");
    test_distances("ZENITH", "ZEИITH");
    test_distances("大犬", "ワルツ第17番 ト短調 \"大犬のワルツ\"");
    test_distances("大犬", "華麗なる大犬円舞曲");
    test_distances("大犬のワルツ", "ワルツ第17番 ト短調 \"大犬のワルツ\"");
    test_distances("大犬のワルツ", "華麗なる大犬円舞曲");
}
