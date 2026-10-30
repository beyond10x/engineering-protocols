//! Acceptance 1 of `story:takt-protocols-builtin` (Atlas ADR 0094 rule 3): takt's three protocols
//! are built-ins of this crate, byte for byte as takt released them at tag 0.12.0.
//!
//! Each file is compared with the Git blob id takt's tree records for it at 0.12.0
//! (`git -C takt rev-parse 0.12.0:protocols/<name>/1.yaml`), computed here from the bytes the
//! registry embeds and from the file in this tree. takt's `protocols/loop-improvement/metrics.yaml`
//! is runtime data, not a protocol, and stays in takt (ADR 0094 rule 2).
//!
//! A built-in differs from takt 0.12.0 only by a change agreed with takt, listed in `AGREED`:
//! its file now has the blob id named there, and undoing the change gives back takt's bytes.

use canon_engineering::registry;

/// A change to a built-in moved from takt that takt agreed to: the protocol, the text takt 0.12.0
/// had, the text that replaces it, and the Git blob id of the file with the change.
/// D257 (2026-10-30): `story.delivery/1`'s `pr.open` names no organisation's delivery route.
const AGREED: [(&str, &str, &str, &str); 1] = [(
    "story-delivery",
    "through the repository's bot route",
    "through the run's forge",
    "bf0a661f119b12393c0254a1c09460dbd07a2e6e",
)];

/// The blob id `name`'s file must have now, and its bytes with every agreed change undone.
fn agreed(name: &str, bytes: &str) -> (Option<&'static str>, String) {
    let mut undone = bytes.to_owned();
    let mut current = None;
    for (protocol, before, after, blob) in AGREED {
        if protocol == name {
            assert_eq!(
                undone.matches(after).count(),
                1,
                "{name}@1 carries the agreed text `{after}` exactly once"
            );
            undone = undone.replace(after, before);
            current = Some(blob);
        }
    }
    (current, undone)
}

/// Each built-in moved from takt, its protocol id, and its Git blob id at takt tag 0.12.0.
const FROM_TAKT: [(&str, &str, &str); 3] = [
    (
        "story-delivery",
        "story.delivery",
        "e21daf63a19b4e72c2abdbd6c01f412303e7b288",
    ),
    (
        "self-improvement",
        "self.improvement",
        "4c477b5a3408cdf20216f9e113af371541c826c4",
    ),
    (
        "loop-improvement",
        "loop.improvement",
        "6d848471c4926a0f5724ecab456eb33980f6e4d2",
    ),
];

/// The Git blob id of `bytes`: SHA-1 over `blob <length>\0` and the bytes.
fn blob_id(bytes: &[u8]) -> String {
    let mut message = format!("blob {}\0", bytes.len()).into_bytes();
    message.extend_from_slice(bytes);
    sha1(&message)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// SHA-1 (FIPS 180-4) of `message`; used only to name a Git blob, as Git does.
fn sha1(message: &[u8]) -> [u8; 20] {
    let mut state: [u32; 5] = [
        0x6745_2301,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ];
    let mut padded = message.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&((message.len() as u64) * 8).to_be_bytes());
    for block in padded.as_chunks::<64>().0 {
        let mut words = [0u32; 80];
        for (index, word) in block.as_chunks::<4>().0.iter().enumerate() {
            words[index] = u32::from_be_bytes(*word);
        }
        for index in 16..80 {
            words[index] =
                (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16])
                    .rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = state;
        for (index, word) in words.iter().enumerate() {
            let (f, k) = match index {
                0..=19 => ((b & c) | (!b & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let next = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = next;
        }
        for (value, add) in state.iter_mut().zip([a, b, c, d, e]) {
            *value = value.wrapping_add(add);
        }
    }
    let mut digest = [0u8; 20];
    for (chunk, value) in digest.as_chunks_mut::<4>().0.iter_mut().zip(state) {
        *chunk = value.to_be_bytes();
    }
    digest
}

#[test]
fn blob_id_is_gits() {
    // `git hash-object` of an empty file and of `hello\n`, and a message longer than one block.
    assert_eq!(blob_id(b""), "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    assert_eq!(
        blob_id(b"hello\n"),
        "ce013625030ba8dba906f756967f9e9ca394464a"
    );
    assert_eq!(
        sha1(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
    );
}

#[test]
fn takt_protocols_are_builtins_byte_for_byte() {
    let listed = registry::list();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (name, id, blob) in FROM_TAKT {
        assert!(
            listed.contains(&(name, 1)),
            "registry::list names {name}@1: {listed:?}"
        );
        let builtin = registry::get(name, 1)
            .unwrap_or_else(|error| panic!("registry::get({name}, 1): {error}"));
        assert_eq!(
            builtin.model.protocol.id.as_str(),
            id,
            "{name}@1 declares protocol id `{id}`"
        );
        assert_eq!(builtin.model.protocol.revision, 1, "{name}@1 is revision 1");
        let (current, undone) = agreed(name, builtin.yaml);
        assert_eq!(
            blob_id(builtin.yaml.as_bytes()),
            current.unwrap_or(blob),
            "registry::get({name}, 1) is takt 0.12.0's protocols/{name}/1.yaml byte for byte, \
             but for the changes AGREED lists"
        );
        assert_eq!(
            blob_id(undone.as_bytes()),
            blob,
            "registry::get({name}, 1) with its agreed changes undone is takt 0.12.0's \
             protocols/{name}/1.yaml byte for byte"
        );
        let path = root.join("protocols").join(name).join("1.yaml");
        let on_disk =
            std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(
            blob_id(&on_disk),
            current.unwrap_or(blob),
            "{} is takt 0.12.0's protocols/{name}/1.yaml byte for byte, but for the changes \
             AGREED lists",
            path.display()
        );
    }
    let metrics = root.join("protocols/loop-improvement/metrics.yaml");
    assert!(
        !metrics.exists(),
        "takt's loop metrics are runtime data and stay in takt (ADR 0094 rule 2)"
    );
}
