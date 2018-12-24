wget -qO- https://github.com/jgthms/bulma/releases/download/0.7.2/bulma-0.7.2.zip | tar xvz -C src/client/vendor
wget https://github.com/FortAwesome/Font-Awesome/releases/download/5.6.1/fontawesome-free-5.6.1-web.zip

unzip fontawesome-free-5.6.1-web.zip -d src/client/vendor

rm -rf bulma-0.7.2.zip
rm -rf fontawesome-free-5.6.1-web.zip

cd src/client && ./optimize.sh src/Main.elm