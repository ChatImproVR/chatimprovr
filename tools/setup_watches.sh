#i3-sensible-terminal bash ./tools/run_server.sh
function term() {
    i3-sensible-terminal --working-directory $1 -e $2 &
}

term plugin/ 'bash ../tools/watch_build.sh'
term plugin2/ 'bash ../tools/watch_build.sh'
term plugin3/ 'bash ../tools/watch_build.sh'
term ./ bash
