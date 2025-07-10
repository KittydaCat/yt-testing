use futures::StreamExt;
use rspotify::{prelude::BaseClient, ClientCredsSpotify, Credentials};
use rustypipe::{client::RustyPipe};

use rspotify::model as sp_model;
use rustypipe::model as yt_model;

#[derive(Debug)]
struct Artist {
    name: String,
    // popularity: u32
}

impl Artist {
    fn from_spotify(artist: sp_model::SimplifiedArtist) -> Self {
        Artist { name: artist.name}
    }

    fn from_youtube(artist: yt_model::ArtistId) -> Self {
        Artist { name: artist.name }
    }
}

#[derive(Debug)]
enum AlbumType {
    Album,
    Single,
    Other(String),
}

impl From<yt_model::AlbumType> for AlbumType {
    fn from(value: yt_model::AlbumType) -> Self {
        match value {
            yt_model::AlbumType::Album => AlbumType::Album,
            yt_model::AlbumType::Ep => AlbumType::Other(String::from("Ep")),
            yt_model::AlbumType::Single => AlbumType::Single,
            yt_model::AlbumType::Audiobook => AlbumType::Other(String::from("Audiobook")),
            yt_model::AlbumType::Show => AlbumType::Other(String::from("Show")),
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
struct Album {
    name: String,
    album_type: AlbumType,
    release_year: Option<u16>,
    artists: Vec<Artist>,
    // total_tracks: u32,
}

impl Album {
    fn from_spotify(album: sp_model::album::SimplifiedAlbum) -> Self {
        
        let release_year = album.release_date.map(|x| x[0..4].parse().unwrap());

        let album_type = match album.album_type.as_ref().map(|x| x.as_str()) {
            None => todo!(),
            // None => AlbumType::Other(String::from("None")),
            Some("album") => Some(AlbumType::Album),
            Some("single") => Some(AlbumType::Single),
            Some("complilation") => {
                Some(AlbumType::Other(album.album_type.clone().unwrap()))
            },
            Some(x) => panic!("{x}"),
        }.expect("Spotify says it should never give us a None");

        let artists = album.artists.into_iter().map(Artist::from_spotify).collect();

        Self {
            name: album.name,
            album_type,
            release_year,
            artists,
        }

    }

    fn from_youtube(album: yt_model::MusicAlbum) -> Self {

        Self {
            name: album.name,
            album_type: AlbumType::from(album.album_type),
            release_year: album.year,
            artists: album.artists.into_iter().map(Artist::from_youtube).collect(),
        }

    }

    fn compare(&self, other: &Self) -> (u32, Vec<Note>) {
        
        let mut score = 0;
        let mut notes = Vec::new();

        if self.name == other.name {
            score += 100;
        } else if false {
            todo!();

            notes.push(Note::AlbumDifferentNamePrefix);
        } else {
            notes.push(Note::AlbumDifferentName);
        }

        

        (score, notes)
    } 
}

#[derive(Debug)]
enum Note {
    // time diff by seconds and percent and the longer source
    LargeTimeDiff(u32, f64, Source),

    DifferentName,
    DifferentNamePrefix,

    AlbumDifferentName,
    AlbumDifferentNamePrefix,

    ExtraArtists(Vec<Artist>),
    MismatchArtists(Vec<Artist>, Vec<Artist>),
    
    AlbumExtraArtists(Vec<Artist>),
    AlbumMismatchArtists(Vec<Artist>, Vec<Artist>),

    AlbumTypeDiff,
    AlbumYearDiff,

    LacksDuration(Source),
}

#[derive(Debug, Clone)]
enum Source {
    Spotify(sp_model::TrackId<'static>),
    Youtube(String),
}

#[derive(Debug)]
struct Song {
    name: String,
    source: Source,
    album: Album,
    artists: Vec<Artist>,
    duration: Option<u32>,
}

impl Song {
    
    fn from_spotify(track: sp_model::FullTrack) -> Self {
        
        let artists = track.artists.into_iter().map(Artist::from_spotify).collect();

        let duration = Some(track.duration.num_seconds() as u32); 

        Song {
            name: track.name,
            source: Source::Spotify(track.id.unwrap()),
            album: Album::from_spotify(track.album),
            artists,
            duration,
        }

    }

    async fn from_youtube(track: yt_model::TrackItem, rp: &RustyPipe) -> Self {

        let artists = track.artists.into_iter().map(Artist::from_youtube).collect();
        
        let album_id = track.album.unwrap();

        let yt_album = rp.query().music_album(album_id.id).await.unwrap();

        Song {
            name: track.name,
            source: Source::Youtube(track.id),
            album: Album::from_youtube(yt_album),
            artists,
            duration: track.duration,
        }

    }

    fn compare(&self, other: &Song) -> (u32, Vec<Note>) {

        const EXACT_NAME_MATCH: u32 = 100;
        // const PREFIX_NAME_MATCH: u32 = 50;
        
        // const SEC_TIME_DIFF: u32 = 5;

        let mut x = 0;
        let mut notes = Vec::new();

        if self.name == other.name {
            x += EXACT_NAME_MATCH;
        } else if false {
            todo!()
        } else {
            // no name match
        }
        
        if let (Some(dur1), Some(dur2)) = (self.duration, other.duration) {
            
            let diff = dur1.abs_diff(dur2);

            let percent_dif = (diff as f64) / (dur1).min(dur2) as f64;

            if diff < 10 {
                
            } else {

                let longer = if dur1 > dur2 {&self.source} else {&other.source};

                notes.push(Note::LargeTimeDiff(diff, percent_dif, longer.clone()));
            }
        }

        (x, notes)
        

    }

    // async fn search_youtube(&self, rp: &RustyPipe) -> impl futures::Stream {
    //     
    //     let vec = rp.query().music_search_tracks(&self.name).await.unwrap()
    //         .items.items;
    //     
    //      futures::stream::iter(vec.into_iter()).map(|x| Song::from_youtube(x, rp))

    // }

    async fn best_yt_match(&self, rp: &RustyPipe) -> Option<Song> {

         let mut vec = rp.query().music_search_tracks(&self.name).await.unwrap()
            .items.items.into_iter();
        
        // let stream = futures::stream::iter(vec.into_iter()).then(|x| Song::from_youtube(x, rp));
        
        let mut best_match = None::<((u32, Vec<Note>), Song)>;

        while let Some(track) = vec.next() {
            
            let song = Song::from_youtube(track, rp).await;

            let cmp = self.compare(&song);

            if let Some((score, _)) = &best_match && score.0 < cmp.0 {

                // TODO look at notes to determine validity

                best_match = Some((cmp, song))
            }

        }
        
        // let iter = vec.into_iter().map(|x| Song::from_youtube(x, rp));
        
        best_match.map(|x| x.1)
    }

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

    let og_tracks = sp_client.playlist_items(playlist, None, None)
        .filter_map(|x| match x.unwrap().track.unwrap() {
            rspotify::model::PlayableItem::Track(full_track) => {
                futures::future::ready(Some(full_track))
            },
            rspotify::model::PlayableItem::Episode(_) => {
                futures::future::ready(None)
            },
        })
        .collect::<Vec<_>>().await;
    

    
}
