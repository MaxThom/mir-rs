#!/bin/sh                                                                                                   

session="mir"

# Check if the session exists
tmux has-session -t $session 2>/dev/null

# If the session exists, kill it
if [ $? -eq 0 ]; then
    tmux attach-session -t $session
    #tmux kill-session -t $session
    exit
fi

tmux new-session -d -s $session

window=0
tmux split-window -h -p 50
tmux split-window -v -p 50
tmux split-window -h -p 75
tmux select-pane -t 2
tmux split-window -h -p 75

tmux select-pane -t 1
tmux send-keys 'make dizer'
tmux select-pane -t 2
tmux send-keys 'make redox'
tmux select-pane -t 3
tmux send-keys 'make rabbit' C-m
tmux select-pane -t 4
tmux send-keys 'make flux'
tmux select-pane -t 5
tmux send-keys 'make db' C-m



#window=1
#tmux new-window -t $session:$window -n 'redox'
#tmux send-keys -t $session:$window 'vim package.json'

#window=2
#tmux new-window -t $session:$window -n 'flux'

#window=3
#tmux new-window -t $session:$window -n 'rabbit'
#tmux send-keys -t $session:$window 'npm run serve'

tmux attach-session -t $session