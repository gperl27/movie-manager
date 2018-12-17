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
    { filename : String, filepath : String, exists : Bool }


type alias Model =
    { movies : List Movie, search : String }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { movies = []
      , search = ""
      }
    , sendSearch ""
    )



-- Update


type Msg
    = ChooseFolder
    | Search String
    | Play Movie
    | JSONData (List Movie)


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
                        ]
                  )
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



-- Subscriptions


subscriptions : Model -> Sub Msg
subscriptions model =
    toFrontEnd decodeValue


decodeValue : JE.Value -> Msg
decodeValue raw =
    case JD.decodeValue movieListDecoder raw of
        Ok movies ->
            JSONData movies

        Err error ->
            JSONData []


view : Model -> Html Msg
view model =
    section [ class "section" ]
        [ div [ class "container" ]
            [ div [ class "columns" ]
                [ div [ class "column", class "is-one-quarter", class "has-text-centered" ] [ button [ class "button", onClick ChooseFolder ] [ text "Choose Folder" ] ]
                , div [ class "column" ]
                    [ input [ class "input", class "is-primary", placeholder "Search", value model.search, onInput Search ] []
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
                                            , div [] [ span [] [ text l.filename ] ]
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
    JD.map3 Movie
        (field "filename" string)
        (field "filepath" string)
        (field "exists" bool)


movieListDecoder : Decoder (List Movie)
movieListDecoder =
    JD.list movieDecoder



-- PORTS


port toBackEnd : String -> Cmd msg


port toFrontEnd : (JE.Value -> msg) -> Sub msg
