# Rebone
### A tool for porting Hitman WoA S1 weightedprims to S2 and S3 bonerigs
#### CLI Usage:
rebone.exe <input_prim> <from_borg> <to_borg> <output_prim>
#### GUI Usage:
![rebone](https://github.com/2kpr/rebone/assets/96332338/2815581b-20a7-4e8e-bd2f-1c0470b0c4ac)
#### Example Output:
```
rebone.exe 001FA83D69D89B24.PRIM 00375A5DF0DE7051.BORG 0017416135CF879C.BORG output.prim

00375A5DF0DE7051.BORG has 231 bones
0017416135CF879C.BORG has 233 bones
Unique bones in 0017416135CF879C.BORG:
  - fa_teeth_dw
  - fa_teeth_up
001FA83D69D89B24.PRIM has 10 meshes
Remapped 33877 joints in 10 meshes
Remapped PRIM file output to output.prim
```
