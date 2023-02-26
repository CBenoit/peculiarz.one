# Premier pain à 80% d’hydratation

Farine de gluten: 65g (72% de protéines)
Robin Hood Bread: 395g (13.2% de protéines) 
Folle farine complète: 50g (13.2% de protéines)
Levain 50% hydratation: 150g (75g folle farine) (75ml eau)
Eau ajoutée: 400ml

Baked: 31 janvier à 19h

Pictures:
- 01GSN76YMYJWY2XDDKM4435KM0
- 01GSN780B321AGFTAY8DZJ0PFA
- 01GSN786HT99C0MJ53WTSNPPYZ
- 01GSN78B8JZX4A59X08E3Z44A4

# Standard sourdough bread using all-purpose white flour

Total Weight  Total Flour  Added Flour  Total Water  Added Water  Total Starter  Added Salt
688 g  400 g  347 g  280 ml  253 ml  80 g  8 g

347g de farine blanche cosco
8g de sel kosher
50% hydratation starter

+ room temperature: around 22°C (not fiable)
+ fermentation start at 9h15
+ 1 lamination
+ 4 coil folds with 1 hour interval
+ shaping at 22h00 (two folds and roll technique), got a decent shape, I applied in banneton stitching in order to further increase strength. I’m unsure whether that was a good idea or not.
+ Overnight fridge proofing
+ baked at 10h00 next morning (30 minutes steam, 15-20 minutes without steam)

Very satisfying bread!

bake date: 2023/02/10

Pictures: (none)

# Experimentation: very high hydratation (120%) bread using gluten powder (72% proteins)

Total Weight  Total Flour  Added Flour  Total Water  Added Water  Total Starter  Added Salt
822 g  352 g  321 g  464 ml  448 ml  46 g 7 g

151g de farine blanche cosco 
170g de gluten powder
7g de sel kosher
50% hydratation starter

+ room temperature: around 22°C (not fiable)
+ fermentolyse
+ fermentation start at 19h
+ 1 lamination (I couldn’t stretch as much as desired)
+ 3 coil folds (very elastic, couldn’t really stretch) with 30-45 minutes interval
+ overnight fermentation
+ shaped at 9h30 next morning (two folds and roll). I failed at shaping the bread properly. I tried to apply in-banneton stitching, that WASN’T a great idea, resulted in worse shape
+ Overnight fridge proofing
+ baked at 10h00 next morning

Mie trop élastique et trop humide après la cuisson "standard".
Je pense qu’il faudrait augmenter la durée de cuisson avec vapeur afin d’avoir une meilleure mie.

bake date: 2023/02/11

Pictures:
- 01GSN7QHR19RWJ8YCN57EJKMSP
- 01GSN7QHR1HXGJHP0W852AENAM
- 01GSN7QHR1R9JEK7BEK2FBMYF6
- 01GSN7QHR1ZZEY9JS27Y7R48H3
- 01GSN7QHR10FE4P77KAP2ZKAPV
- 01GSN7QHR1YZCD77S21CJTXVEY

# Pain blanc 85% d’hydratation

DoughProblem {
            mass: Target::free(),
            flour: Target::free(),
            wheat_proteins: Target::by_ratio(Ratio::new::<ratio>(0.14)),
            hydratation: Ratio::new::<ratio>(0.85),
            salt_ratio: Ratio::new::<ratio>(0.02),
            ingredients: vec![
                (cosco_flour(), Target::free()),
                (gluten_powder(), Target::free()),
                (
                    stiff_sourdough_starter(),
                    Target {
                        mass: Some(Mass::new::<gram>(95.)),
                        ratio: Some(Ratio::new::<ratio>(0.2)),
                    },
                ),
                (tap_water(), Target::free()),
                (table_salt(), Target::free()),
            ],
        };

        // cosco flour (11.5% proteins): 395g
        // gluten powder: 17g
        // starter: 95g
        // water: 372ml

Bulk fermentation :9h
1 lamination 
6 coil folds 
Shaping : 18h30
Overnight fridge proofing
Baked : le 15 février à 17h

Keeps its shape nicely 

Crumb suggests under fermentation.

Pictures:
- 01GSN8090Q1QF5CXXHM1PJ9YRX
- 01GSN8090QMMED6S1K19CK4901
- 01GSN8090QS78BB5MCVMJQ0NH3
- 01GSN8090QQZHZ2E8J6V3PZY9V

# Pain blanc ?% d’hydratation

added_flour = 328g  
gluten_powder = 7g
starter = 79g
added_water = 305g
salt = 20g

fermentation: 10h
1 lamination 
6 coil folds 
Shaping : 23h59
No in banneton stitching
Overnight fridge proofing
Baked : le 20 février à ?h

J’ai mis trop de sel par accident, ça se sentait dans le goût.
Un pain satisfaisant autrement.

Pictures:
- …

# Une miche et deux baguettes à 80% d’hydratation

| Total Mass := 1501.5 g
| Flour := 825.0 g
| Water := 660.0 g (80.0 %)
| Wheat Proteins := 115.5 g (14.0 %)
| Ingredients ⤵
0 - White flour (702.9 g)
  | ID := 01GSRZXQJ113V5Q7M3WFWHYFDR
  | Added By := 00000000000000000000000000
  | Category := Flour
  | Kind := WhiteFlourUnbleached
  | Proteins := 13.0 %
  | Ash := 6.0 %
1 - Gluten powder (12.1 g)
  | ID := 01GSRZXQJ20RXQT7255AXFMEP7
  | Added By := 00000000000000000000000000
  | Category := Flour
  | Kind := GlutenPowder
  | Proteins := 72.0 %
  | Ash := 6.0 %
2 - Bobby the Stiff Sourdough Starter (165.0 g)
  | ID := 01GSRZXQJ2TFRA54GTC93V0J0F
  | Added By := 00000000000000000000000000
  | Category := Leavener
  | Kind := SourdoughStarter
  | Proteins := 9.3 %
  | Ash := 4.0 %
  | Water := 33.3 %
  | Hydratation := 50.0 %
3 - Dechlorinated tap water (605.0 g)
  | ID := 01GSRZXQJ2E957TNAN8JJQ4MKZ
  | Added By := 00000000000000000000000000
  | Category := Liquid
  | Kind := Water
  | Water := 100.0 %
4 - Table salt (16.5 g)
  | ID := 01GSRZXQJ2YZ87H0Y6PBHAV3NW
  | Added By := 00000000000000000000000000
  | Category := Salt
  | Kind := TableSalt
  | Salt := 100.0 %

fermentation started: 16h50
1 lamination 
6 coil folds
Overnight fermentation
Shaping next morning: 09h00
Fridge proofing
Baked at: 16h10

Deux excellentes baguettes.
La miche n’était cependant pas assez cuite. Très triste.

Pictures:
- …

# Pain blanc à 80% d’hydratation

| Total Mass := 773.5 g
| Flour := 425.0 g
| Water := 340.0 g (80.0 %)
| Wheat Proteins := 59.5 g (14.0 %)
| Ingredients ⤵
0 - White flour (362.1 g)
  | ID := 01GT27H6EM5CH19GVJDZWC0DGY
  | Added By := 00000000000000000000000000
  | Category := Flour
  | Kind := WhiteFlourUnbleached
  | Proteins := 13.0 %
  | Ash := 6.0 %
1 - Gluten powder (6.2 g)
  | ID := 01GT27H6ENFGD19EG71H0PAS2M
  | Added By := 00000000000000000000000000
  | Category := Flour
  | Kind := GlutenPowder
  | Proteins := 72.0 %
  | Ash := 6.0 %
2 - Bobby the Stiff Sourdough Starter (85.0 g)
  | ID := 01GT27H6ENPBW9G36FHJ8YAS3G
  | Added By := 00000000000000000000000000
  | Category := Leavener
  | Kind := SourdoughStarter
  | Proteins := 9.3 %
  | Ash := 4.0 %
  | Water := 33.3 %
  | Hydratation := 50.0 %
3 - Dechlorinated tap water (311.7 g)
  | ID := 01GT27H6EN85EW13WKP2M1V2BE
  | Added By := 00000000000000000000000000
  | Category := Liquid
  | Kind := Water
  | Water := 100.0 %
4 - Table salt (8.5 g)
  | ID := 01GT27H6ENH9MEP0J221CYDFG0
  | Added By := 00000000000000000000000000
  | Category := Salt
  | Kind := TableSalt
  | Salt := 100.0 %

fermentation started: 12h30
1 lamination 
6 coil folds
Shaping at 5h00 (17 hours later)
With in banneton stitching
Fridge proofing
Baked next day: 10h00

Pictures:
- …

# Trois baguettes à 80% d’hydratation

| Total Mass := 825.1 g
| Flour := 453.3 g
| Water := 362.7 g (80.0 %)
| Wheat Proteins := 63.5 g (14.0 %)
| Ingredients ⤵
0 - White flour (401.1 g)
  | ID := 01GT5WFMHZHKS5H7N5SFE0HZFY
  | Added By := 00000000000000000000000000
  | Category := Flour
  | Kind := WhiteFlourUnbleached
  | Proteins := 13.0 %
  | Ash := 6.0 %
1 - Gluten powder (6.9 g)
  | ID := 01GT5WFMJ0VDG3T6ZYKQBWRP76
  | Added By := 00000000000000000000000000
  | Category := Flour
  | Kind := GlutenPowder
  | Proteins := 72.0 %
  | Ash := 6.0 %
2 - Bobby the Stiff Sourdough Starter (68.0 g)
  | ID := 01GT5WFMJ09PC93SBGBMTRN7XZ
  | Added By := 00000000000000000000000000
  | Category := Leavener
  | Kind := SourdoughStarter
  | Proteins := 9.3 %
  | Ash := 4.0 %
  | Water := 33.3 %
  | Hydratation := 50.0 %
3 - Dechlorinated tap water (340.0 g)
  | ID := 01GT5WFMJ0FBABKFEV3H290VN7
  | Added By := 00000000000000000000000000
  | Category := Liquid
  | Kind := Water
  | Water := 100.0 %
4 - Table salt (9.1 g)
  | ID := 01GT5WFMJ0PF0V5D31PZYYF3RP
  | Added By := 00000000000000000000000000
  | Category := Salt
  | Kind := TableSalt
  | Salt := 100.0 %

Fermentation start: 22h00
1 lamination
1 stretch and fold
Overnight fermentation
Shaping at …
With in banneton stitching
Fridge proofing
Baked next day: …

Pictures:
- …
