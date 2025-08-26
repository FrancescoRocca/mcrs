use serde::Serialize;

#[derive(Serialize)]
struct Version {
    name: String,
    protocol: u32,
}

#[derive(Serialize)]
struct PlayerSample {
    name: String,
    id: String,
}

#[derive(Serialize)]
struct Players {
    max: u32,
    online: u32,
    sample: Vec<PlayerSample>,
}

#[derive(Serialize)]
struct Description {
    text: String,
}

#[derive(Serialize)]
pub struct Status {
    version: Version,
    players: Players,
    description: Description,
    favicon: String,
    #[serde(rename = "enforcesSecureChat")]
    enforces_secure_chat: bool,
}

impl Status {
    pub fn new(
        name: &str,
        protocol: u32,
        max_players: u32,
        online_players: u32,
        description_text: &str,
    ) -> Self {
        let version = Version {
            name: name.to_string(),
            protocol,
        };

        let player_sample = PlayerSample {
            name: "I'm coded in Rust!".to_string(),
            id: "0541ed27-7595-4e6a-9101-6c07f879b7b5".to_string(),
        };

        let players = Players {
            max: max_players,
            online: online_players,
            sample: vec![player_sample],
        };

        let description = Description {
            text: description_text.to_string(),
        };

        let favicon = "data:image/png;base64,
	    iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAACXBIWXMAACdeAAAnXgHPwViOAAAAGXRFWHRTb2Z0d2FyZQB3d3cuaW5rc2NhcG
	    Uub3Jnm+48GgAAA1dJREFUeJztm8FPE0EUxr+
	    3u6BE8KAJQZCbisbEgsSEpi0pqEeDF4hcTEy8Ef8EEv4Iw8UYTfSANpGEK1EaaEOM0UIjMcBNCKKJB4VTy87zQFta0t1u2WUH2/
	    mdtvte3nzvy3Q6u9sFFPUNWQV6I5ELpjB6QHTGT0FeQ4xdo0GkPsXj22Xjh08EgoMdZGASjHvl4v8pAsAMsT6WSs5uFQdKGgwEBzs0nRcZ1Omr
	    PJ8g8AbY6Cs2QStJMDBZq80DAIM6mcynxecKM6AndLedydxE7Ux7K0SDITrya0LRDDADqP3mAUDLmBQofMgfMKFZjh7/
	    Iaaz+WPNLrEeUAbIFiAbZYBsAbJRBsgWIBvDbQFienyp/dzLWCxmWuVcj0abG7L6axAPOSw7nTXEw5V4fNcqYXh4WF/f/
	    v0IjGdViy7C9Qyo1DwArMTju6RhymlNZnpj1zwAxGIx83Lb+RdOa1rh2oBKzedhZkd5AEDEwsux7aj7NUAZIFuAbJQBsgXIRhngtkBXKNTiJI/
	    BjvL2c53dnHE6th2ud4JNOP2qJ3J7yu53nsEtxDQOoiWA1yuUvELg8UB4gAi0Y5VERDqDR8FHlg7AAwNAPMQM2y0ugQDC8lIi0gtMVNjkTGjd4
	    fkvBDy3y2J22XkOP9eAtcrNA7mctWNXk0MtgrIFyKbuDXC/
	    CB7wSzD3p5Nzq24LLSU+jFjFboQGujSieQCtbscBPJwBBCS8aL4S6eTcKgEJr+p5ZgADrq/
	    NZYxV92uAMkC2ANkoA2QLkI0yQLYA2Xi5E7TlRjR6UdvTgk5yhSEW0/
	    H45nFrAnw0INf8W4e5IwBix6soN5Yfg5xklAFeFSJAt40T2caLYSZbXZXGqgYvL4bCvcHo1XKxrlCohcGjTmsR8QOrO743++
	    9cYyByVJ2H8XIRbDV17Vt3eLB8tLp7mPeb6NTfcrWEcPTg2DFqDZAtQDbKANkCZKMMkC1ANsqAgwPrJ7G1Bgv8yR8XDCDKLGP/
	    7apaRzQ2inT+Q8GAzwsLPwDMSJHkI8w0XfwOYelbY6yPEXjDf1m+8d3QMk+
	    KT5QYkErOboGNPgDTqK2vgyDgnU7ZvtxML2D5ltitaLTN3NO7RRX/7TmJaKCdTBaprx/f/5StRXES+Qfq2fZ/
	    nG9J9gAAAABJRU5ErkJggg==";

        Status {
            version,
            players,
            description,
            favicon: favicon.to_string(),
            enforces_secure_chat: true,
        }
    }

    pub fn json(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}
