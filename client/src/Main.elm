port module Main exposing (main)

import Browser
import Html exposing (..)
import Html.Events exposing (..)



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
    { file : String }


type alias Model =
    { movies: List Movie }


init : () -> ( Model, Cmd Msg )
init _ =
    ( Model []
    , Cmd.none
    )



-- Update


type Msg
    = ChooseFolder


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ChooseFolder ->
            ( model, toBackEnd "openFolder" )



-- Subscriptions


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none
    
-- VIEW


view : Model -> Html Msg
view model =
    div []
        [ button [ onClick ChooseFolder ] [ text "Choose Folder" ]
        ]

port toBackEnd : String -> Cmd msg