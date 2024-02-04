# Driver for Analog Devices AD57xx series of dual and quad channel 16/14/12bit DACs

For now only the AD57x4 quad channel chips are supported. Readback operation is currently
untested as my hardware does not support it. If you are in the opportunity to do so please
let me know.

Any contribution to this crate is welcome, as it's my first published crate any 
feedback is appreciated.

## Done:
 - Register definitions
 - Write functionality for all registers
 - Minimal working example with a shared bus on stm32f4xx


## To-do:
 - #[tests] on target
 - Testing readback functionality
 - Exclusive device struct
 - Dual channel chip support
 - Support daisy-chain operation
 - Async support

