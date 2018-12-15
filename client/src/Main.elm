port module Main exposing (main)

import Browser
import Html exposing (..)
import Html.Attributes exposing (..)
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
    { movies: List Movie, search: String } 


init : () -> ( Model, Cmd Msg )
init _ =
    ( {
        movies = [],
        search = ""
    } 
    , Cmd.none
    )



-- Update


type Msg
    = ChooseFolder
    | Search String
    | JSONData (List Movie)

sendOpenFolder: Cmd Msg
sendOpenFolder =
    let
        json = JE.object [("_type", JE.string "OpenFolder")]
        str = JE.encode 0 json
    in
        toBackEnd str

sendSearch: String -> Cmd Msg
sendSearch keyword =
    let
        json = JE.object [("_type", JE.string "Search"),
                          ("keyword", JE.string keyword)
                         ]
                                                       
        str = JE.encode 0 json
    in
        toBackEnd str
    


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ChooseFolder ->
            ( model, sendOpenFolder )
        Search str->
            ({ model | search = str}, sendSearch str )
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
        --   button [ onClick Search ] [ text "search" ],
          input [ placeholder "Text to reverse", value model.search, onInput Search ] [],
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