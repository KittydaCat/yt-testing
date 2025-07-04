use futures::StreamExt;
use rspotify::{prelude::BaseClient, ClientCredsSpotify, Credentials};
use rustypipe::{client::RustyPipe};

use rspotify::model as sp_model;
use rustypipe::model as yt_model;

#[derive(Debug)]
enum AlbumType {
    Album,
    Single,
    Other(String),
}

#[derive(Debug)]
struct Album {
    name: String,
    album_type: Option<AlbumType>,
    release_year: Option<u16>,
    artists: Vec<Artist>,
}

#[derive(Debug)]
struct Popularity {

}

#[derive(Debug)]
struct Artist {
    name: String,
    popularity: Option<Popularity>,
}

#[derive(Debug)]
struct Song {
    name: String,
    album: Album,
    artists: Vec<Artist>,
    duration: Option<u32>,
}

impl Song {
    
    // fn from_spotify(track: sp_model::FullTrack) -> Self {
    //     
    //     Song {
    //         name: track.name,
    //         album: Some(track.album.name),
    //         artists: track.artists.into_iter()
    //             .map(|x| x.name).collect(),
    //     }

    // }

    // fn from_youtube(track: yt_model::TrackItem) -> Self {
    //     
    //     Song {
    //         name: track.name,
    //         album: track.album.map(|x|x.name),
    //         artists: track.artists.into_iter().map(|x| x.name).collect()
    //     }

    // }

}

#[tokio::main]
async fn main() {

    dotenvy::dotenv().unwrap();
    
    let rp = RustyPipe::new();
    
    let sp_client = ClientCredsSpotify::new(Credentials::from_env().unwrap());

    sp_client.request_token().await.unwrap();

    let playlist = sp_model::PlaylistId::from_id("1VhaQKU3TNk1EFfOtCtCbD").unwrap();

    // let user = UserId::from_id("uamt3pcjb42o2aanulou96ip1").unwrap();

    // let playlist = client.user_playlist(user, Some(playlist), None).await.unwrap();

    // println!("{songs:?}");
    
    // dbg!(playlist.tracks.items.len());

    let mut tracks = sp_client.playlist_items(playlist, None, None)
        .filter_map(|x| match x.unwrap().track.unwrap() {
            rspotify::model::PlayableItem::Track(full_track) => {
                futures::future::ready(Some(full_track))
            },
            rspotify::model::PlayableItem::Episode(_) => {
                futures::future::ready(None)
            },
        });

    while let Some(track) = tracks.next().await {
        // we know from spotify:
        // album
        // artists
        //   Type - single or not
        //   name 
        //   artists
        //   label
        // disc_number?
        // duration
        // explict?
        // name
        
        // we can search
        // -by string only
        // artist
        // album 
        // tracks
        
        let yt_tracks_result = rp.query().music_search_tracks(&track.name).await.unwrap();
        
        let yt_tracks = &yt_tracks_result.items.items;

        // dbg!(yt_tracks);

        let mut matches: Vec<(&yt_model::TrackItem, usize)> = Vec::new();

        for yt_track in yt_tracks {

            let mut score = 0_usize;

            if yt_track.name == track.name {
                score += 10;
            }
            
            if yt_track.album.as_ref().unwrap().name == track.album.name {
                score += 5;
            }

            if let Some(id) = yt_track.artist_id.as_ref() {

                let main_artist = rp.query().music_artist(id, false).await.unwrap().name;

                if track.artists.iter().any(|x| x.name == main_artist) {
                    score += 5;
                }

            }

            matches.push((yt_track, score));
        }

        let best_match = matches.iter().max_by_key(|x| x.1).unwrap();

        println!("{} - {} - https://www.youtube.com/watch?v={}", track.name, best_match.1, best_match.0.id);

    }

}
