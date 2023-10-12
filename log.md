First thing i did was install the driver from https://github.com/RASPIAUDIO/WM8960-Audio-HAT
then i assumed it was broken becuase the wm8960-soundcard.service failed.

Didnt know how to test it at this time.

next then I installed the driver from waveshare and after looking at the code i saw it had more recent updates. link is here git clone git@github.com:waveshare/WM8960-Audio-HAT.git. I still recieved a similiar error to before about wm8960-soundcard.service failed. 

i need to run some test.

I then found this adafruit webpage to test audio. https://learn.adafruit.com/usb-audio-cards-with-a-raspberry-pi/testing-audio

i used `speaker-test -c2` and i heard sound.

then i used `speaker-test -c2 --test=wav -w /usr/share/sounds/alsa/Front_Center.wav` and i heard voices from the speaker. 

NICE!!

rebooted the pi to make sure change stuck and they did
