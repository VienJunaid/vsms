# vsms

Everything else depends on *control-core* and *ui-dashboard* 

Concrete order within that file:

1. Fix the three ENCODED_LEN constants (just arithmetic on the field sizes)
2. VitalsSample::encode/decode — get the roundtrip test passing first
3. AlarmEvent and ConfigUpdate encode/decode — same pattern, repetition will cement it
4. FrameType::from_u8 and AlarmLevel::from_u8 — trivial matches
5. checksum
6. encode_frame
7. decode_frame — save this for last, it's the trickiest (the Incomplete/Invalid/Frame decision tree)
8. Tests as you go, not all at the end — write roundtrip_vitals_sample before you even touch encode_frame