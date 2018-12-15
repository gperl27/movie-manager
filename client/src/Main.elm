port module Main exposing (main)

import Browser
import Html exposing (..)
import Html.Events exposing (..)
import Json.Decode as JD exposing (field, Decoder, int, string)
import Json.Encode as JE exposing (Value)
-- Main


main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }



-- Model


type alias Movie =
    { filename : String, filepath : String }


type alias Model =
    { movies: List Movie } 


init : () -> ( Model, Cmd Msg )
init _ =
    ( {
        movies = []
    } 
    , Cmd.none
    )



-- Update


type Msg
    = ChooseFolder
    | JSONData (List Movie)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ChooseFolder ->
            ( model, toBackEnd "openFolder" )
        JSONData data ->
            ({ model | movies = data }, Cmd.none )        



-- Subscriptions



subscriptions : Model -> Sub Msg
subscriptions model =
    toFrontEnd (decodeValue) 
    
decodeValue : JE.Value -> Msg
decodeValue raw =
    case JD.decodeValue movieListDecoder raw of
        Ok movies ->
            JSONData movies
        Err error ->
            JSONData []



-- VIEW


view : Model -> Html Msg
view model =
    div []
        [ button [ onClick ChooseFolder ] [ text "Choose Folder" ],
          ul [] (List.map (\l -> li [] [ text l.filename ]) model.movies)
        ]

movieDecoder : Decoder Movie
movieDecoder =
    JD.map2 Movie
        (field "filename" string)
        (field "filepath" string)


movieListDecoder : Decoder (List Movie)
movieListDecoder =
    JD.list movieDecoder


port toBackEnd : String -> Cmd msg
port toFrontEnd : (JE.Value -> msg) -> Sub msg