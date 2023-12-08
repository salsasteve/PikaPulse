import numpy as np
import scipy.io.wavfile as wavfile
import matplotlib.pyplot as plt

def apply_hann_window_to_wav(file_path, output_path):
    # Read the WAV file
    sample_rate, data = wavfile.read(file_path)

    # Generate a Hann window
    window_length = len(data)
    hann_window = 0.5 * (1 - np.cos(2 * np.pi * np.arange(window_length) / (window_length - 1)))

    # Apply the Hann window to the data
    windowed_data = data * hann_window[:, np.newaxis]

    # Write the windowed data to a new WAV file
    wavfile.write(output_path, sample_rate, windowed_data.astype(data.dtype))

    # Optional: Plot the original and windowed signal
    plt.figure(figsize=(12, 6))
    plt.subplot(2, 1, 1)
    plt.plot(data)
    plt.title("Original Signal")
    plt.subplot(2, 1, 2)
    plt.plot(windowed_data)
    plt.title("Signal After Applying Hann Window")
    plt.tight_layout()
    plt.show()

# Path to the input WAV file and output file
input_wav_file = '../oli.wav'
output_wav_file = 'output.wav'

# Apply the Hann window to the WAV file
apply_hann_window_to_wav(input_wav_file, output_wav_file)
