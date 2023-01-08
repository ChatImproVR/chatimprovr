#i3-sensible-terminal bash ./tools/run_server.sh
function term() {
    i3-sensible-terminal --working-directory $1 -e bash --init-file ../tools/watch_build.sh &
}

term plugin/
term plugin2/
term plugin3/ 
#./tools/run_server.sh
