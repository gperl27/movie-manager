port module Main exposing (main)

import Browser
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Json.Decode as JD exposing (Decoder, bool, field, int, string)
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
    { filename : String, filepath : String, exists : Bool, folder : String }


type alias Folder =
    { name : String, isChosen : Bool }


type alias Model =
    { movies : List Movie, search : String, folders : List String, chosenFolders : List String }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { movies = []
      , search = ""
      , folders = []
      , chosenFolders = []
      }
    , sendSearch ""
    )



-- Update


type Msg
    = ChooseFolder
    | Search String
    | Play Movie
    | JSONData (List Movie)
    | UpdateFolders (List String)
    | ClickFolder String
    | UnclickFolder String
    | UpdateChosenFolders (List String)


sendOpenFolder : Cmd Msg
sendOpenFolder =
    let
        json =
            JE.object [ ( "_type", JE.string "OpenFolder" ) ]

        str =
            JE.encode 0 json
    in
    toBackEnd str


sendSearch : String -> Cmd Msg
sendSearch keyword =
    let
        json =
            JE.object
                [ ( "_type", JE.string "Search" )
                , ( "keyword", JE.string keyword )
                ]

        str =
            JE.encode 0 json
    in
    toBackEnd str


sendPlayMovie : Movie -> Cmd Msg
sendPlayMovie movie =
    let
        json =
            JE.object
                [ ( "_type", JE.string "Play" )
                , ( "movie"
                  , JE.object
                        [ ( "filename", JE.string movie.filename )
                        , ( "filepath", JE.string movie.filepath )
                        , ( "exists", JE.bool movie.exists )
                        , ( "folder", JE.string movie.folder )
                        ]
                  )
                ]

        str =
            JE.encode 0 json
    in
    toBackEnd str


sendClickFolder : String -> Cmd Msg
sendClickFolder folder =
    let
        json =
            JE.object
                [ ( "_type", JE.string "ClickFolder" )
                , ( "folder", JE.string folder )
                ]

        str =
            JE.encode 0 json
    in
    toBackEnd str


sendUnclickFolder : String -> Cmd Msg
sendUnclickFolder folder =
    let
        json =
            JE.object
                [ ( "_type", JE.string "UnclickFolder" )
                , ( "folder", JE.string folder )
                ]

        str =
            JE.encode 0 json
    in
    toBackEnd str


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ChooseFolder ->
            ( model, sendOpenFolder )

        Search str ->
            ( { model | search = str }, sendSearch str )

        Play movie ->
            ( model, sendPlayMovie movie )

        JSONData data ->
            ( { model | movies = data }, Cmd.none )

        UpdateFolders folders ->
            ( { model | folders = folders }, Cmd.none )

        ClickFolder folder ->
            ( model, sendClickFolder folder )

        UnclickFolder folder ->
            ( model, sendUnclickFolder folder )

        UpdateChosenFolders folders ->
            ( { model | chosenFolders = folders }, Cmd.none )



-- Subscriptions


subscriptions : Model -> Sub Msg
subscriptions model =
    toFrontEnd decodeValue


movieListToMsg : JE.Value -> Msg
movieListToMsg raw =
    case JD.decodeValue (JD.field "movies" movieListDecoder) raw of
        Ok movies ->
            JSONData movies

        Err error ->
            JSONData []


decodeValue : JE.Value -> Msg
decodeValue raw =
    let
        object_type =
            JD.decodeValue (JD.field "data" JD.string) raw
    in
    case object_type of
        Ok "Search" ->
            movieListToMsg raw

        Ok "OpenFolder" ->
            movieListToMsg raw

        Ok "Folders" ->
            case JD.decodeValue (JD.field "folders" (JD.list string)) raw of
                Ok folders ->
                    UpdateFolders folders

                Err error ->
                    UpdateFolders []

        Ok "ChosenFolders" ->
            case JD.decodeValue (JD.field "chosen_folders" (JD.list string)) raw of
                Ok folders ->
                    UpdateChosenFolders folders

                Err error ->
                    UpdateChosenFolders []

        Ok unknown_type ->
            JSONData []

        Err error ->
            JSONData []



-- VIEW


computeFolders : List String -> List String -> List Folder
computeFolders allFolders chosenFolders =
    List.map (\x -> { name = x, isChosen = List.member x chosenFolders }) allFolders


view : Model -> Html Msg
view model =
    section [ class "section" ]
        [ div [ class "container" ]
            [ div [ class "columns" ]
                [ div [ class "column", class "is-one-quarter", class "has-text-centered" ] [ button [ class "button", onClick ChooseFolder ] [ text "Choose Folder" ] ]
                , div [ class "column" ]
                    [ div [ class "columns", class "is-centered", class "is-multiline" ]
                        (List.map
                            (\l ->
                                div [ class "column", class "is-narrow" ]
                                    [ button
                                        [ class "button"
                                        , classList [ ( "is-primary", l.isChosen ) ]
                                        , onClick
                                            (if l.isChosen then
                                                UnclickFolder l.name

                                             else
                                                ClickFolder l.name
                                            )
                                        ]
                                        [ text l.name ]
                                    ]
                            )
                            (computeFolders model.folders model.chosenFolders)
                        )
                    , input [ class "input", class "is-primary", placeholder "Search", value model.search, onInput Search ] []
                    , div [ class "content" ]
                        [ ul [ class "unstyled" ]
                            (List.map
                                (\l ->
                                    li []
                                        [ div [ class "columns", class "is-vcentered" ]
                                            [ div [ class "column", class "is-one-quarter" ]
                                                [ button [ class "button", class "is-primary", onClick (Play l), disabled (not l.exists) ]
                                                    [ span [ class "icon" ]
                                                        [ i [ class "fas", class "fa-play" ] [] ]
                                                    , span [] [ text "Play" ]
                                                    ]
                                                ]
                                            , div [] [ span [] [ text l.filename ], text " - ", span [] [ text l.folder ] ]
                                            ]
                                        ]
                                )
                                model.movies
                            )
                        ]
                    ]
                ]
            ]
        ]


movieDecoder : Decoder Movie
movieDecoder =
    JD.map4 Movie
        (field "filename" string)
        (field "filepath" string)
        (field "exists" bool)
        (field "folder" string)


movieListDecoder : Decoder (List Movie)
movieListDecoder =
    JD.list movieDecoder



-- PORTS


port toBackEnd : String -> Cmd msg


port toFrontEnd : (JE.Value -> msg) -> Sub msg
