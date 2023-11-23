#!/bin/bash

# Extracting playback (speaker) device number
playback_device=$(aplay -l | grep 'card [0-9]:' | grep 'wm8960soundcard' | head -n 1 | awk '{print $2}' | tr -d ':')

# Extracting capture (mic) device number
capture_device=$(arecord -l | grep 'card [0-9]:' | grep 'wm8960soundcard' | head -n 1 | awk '{print $2}' | tr -d ':')

# Creating the .asoundrc configuration
echo "pcm.!default {
  type asym
  capture.pcm \"mic\"
  playback.pcm \"speaker\"
}
pcm.mic {
  type plug
  slave {
    pcm \"hw:$capture_device,0\"
  }
}
pcm.speaker {
  type plug
  slave {
    pcm \"hw:$playback_device,0\"
  }
}" > ~/.asoundrc

echo "The .asoundrc file has been created."

