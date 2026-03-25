import struct
import sys
import numpy as np

def validate_signal(filename, expected_rate=48000, expected_seconds=5):
    with open(filename, 'rb') as f:
        header_data = f.read(64)
        if len(header_data) < 64:
            print(f"Error: Header too short ({len(header_data)} bytes)")
            return

        # Header: magic (4), sequence (8), sample_rate (4), bit_depth (4), pad (44)
        magic = header_data[0:4]
        if magic != b'DP32':
            print(f"Error: Invalid magic identifier. Expected DP32, got {magic}")
            return
        
        sequence, sample_rate, bit_depth = struct.unpack('<QII', header_data[8:24])
        print(f"Header Validated:")
        print(f"  Magic: {magic.decode()}")
        print(f"  Sequence: {sequence}")
        print(f"  Sample Rate: {sample_rate} Hz")
        print(f"  Bit Depth: {bit_depth}-bit")

        if sample_rate != expected_rate:
            print(f"Warning: Sample rate mismatch. Expected {expected_rate}, got {sample_rate}")
        
        # Read samples
        sample_data = f.read()
        num_samples = len(sample_data) // 4
        print(f"Samples read: {num_samples}")
        
        expected_samples = expected_rate * expected_seconds
        if num_samples != expected_samples:
            print(f"Error: Sample count mismatch. Expected {expected_samples}, got {num_samples}")

        # Convert to i32 numpy array
        samples = np.frombuffer(sample_data, dtype='<i4')
        
        # Check range (ignore first 1000 samples for transitent)
        if len(samples) > 1000:
            stable_samples = samples[1000:]
        else:
            stable_samples = samples

        max_val = np.max(stable_samples)
        min_val = np.min(stable_samples)
        print(f"Signal Statistics (Stable):")
        print(f"  Min Value: {min_val}")
        print(f"  Max Value: {max_val}")
        
        # Check for non-zero signal
        if max_val == 0 and min_val == 0:
            print("Error: Signal is entirely zero!")
            return

        print("\nSamples 0-200 (i32 raw):")
        for i in range(0, 200, 10):
            print(f"{i:3}: {samples[i:i+10]}")

        # Check for periodicity
        # Cross-correlation with itself
        if len(stable_samples) > 2000:
            corr = np.correlate(stable_samples[:1000], stable_samples[1:1001], mode='full')
            # Look for peaks
            # (Just a simple check)
            pass

if __name__ == "__main__":
    validate_signal('signal.raw')
