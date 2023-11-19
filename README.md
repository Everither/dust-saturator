# Dust Saturator

A distortion plugin with a very unique distortion algorithm. Adding upper harmonics to the sound source while introducing a sandy texture. Fully preserves the dynamics of the sound (i.e. peaks of the waveform does not get clamped at all).

Built with the [NIH-plug](https://github.com/robbert-vdh/nih-plug) framework and the [VIZIA](https://github.com/vizia/vizia) UI library.

![Dust Saturator Cover](https://github.com/Everither/dust-saturation/assets/122586326/f42a1910-2130-4296-8e5c-1ca55c573f06)

## How It Works

The amplitude of each individual sample point determines how far it gets forward time shifted.

For example, an array of 10 sample points gets processed by the algorithm, with indexes 0 to 9.
Each sample has an amplitude between -1 and 1.

To process the first sample (the sample at index 0) the absolute value of its amplitude (basically dropping the negative sign if there is one) is multiplied by some constant (determined by the parameter `Amount`) then rounded down to an integer.

Suppose the first sample (the sample at index 0) has amplitude 0.3, and `Amount` is set to 10, then the calculated value is 0.3*10=3.
Take 3 steps from this sample, arriving at the fourth sample (sample at index 3). The first sample is then assigned the amplitude of the fourth sample.

## Parameters
`Amount` The maximum time shifting distance, measured in samples.

`Curve` The curve/tension in the time shifting calculation.

- At 1.0, it behaves as a linear line, meaning more dynamics, shortened duration of distortion
- At 0.5, it behaves as a square root curve, meaning less dynamics, longer distortion

## Setup
Currently, this plugin assumes fixed size buffers. Please ensure your DAW is using fixed size buffers.

In FL Studio, this is done by going to:

VST wrapper settings > Troubleshooting > Use fixed sized buffers 

and enabling it.

![image](https://github.com/Everither/dust-saturation/assets/122586326/9eaeeb81-dd1c-475d-b329-a613cbeda0bc)
