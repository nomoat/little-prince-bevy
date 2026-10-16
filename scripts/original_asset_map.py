"""Explicit links between the gameplay manifest and the user's Unity assets."""

MODELS = {}


def model(key, chapter, *names, **options):
    MODELS['model.' + key] = dict(chapter=chapter, names=list(names), **options)


model('prince', 'shared', 'little_prince_body', 'little_prince_scarf', height=1.2)
model('b612.rose', 'b612', 'Rose_Stem', 'Rose_Head_VC', height=.55,
      aliases={'Rose_Head': 'part:rose_head'})
model('b612.cloche', 'b612', 'Cloche_Base', 'Cloche_Glass', size=[.46, .65, .46])
model('b612.boulder', 'b612', 'Planet_ClocheBoulder_LOD0', size=[1.12, .555, 1.12])
model('b612.screen', 'b612', 'WindScreen', height=.75)
model('b612.can', 'b612', 'WateringCan', height=.3)
model('b612.shed', 'b612', 'ToolShed', size=[2.4, 1.8, 1.8])
model('b612.volcano', 'b612', 'VolcanoA_LOD0', height=.9)
model('b612.crater', 'b612', 'VolcanoB_LOD0', height=.5)
model('b612.sprout', 'b612', 'BaobabSeedling', height=.3)
model('b612.pad', 'b612', 'SleepingPad', 'Blanket')
model('b612.stone', 'b612', 'FlatStone_01', height=.18)
model('b612.slab', 'b612', 'Planet_FetchLedge', height=.12)
model('b612.chair', 'b612', 'SunsetChair', height=.98)

for key, name, height in [
    ('rock', 'Rock_01_LOD0', .7), ('rock_small', 'Pebble_05', .24),
    ('rock_large', 'Rock_03_LOD0', 1.2), ('stone_tall', 'Rock_05_LOD0', 1.5),
    ('bush', 'Shrub_Round_LOD0', .6), ('grass_tuft', 'Grass_Short_A_LOD0', .32),
    ('grass', 'Grass_Meadow_A_LOD0', .4), ('flower', 'Flower_Buttercup_A', .35),
    ('flower_red', 'Flower_Campion_A', .35), ('flower_yellow', 'Flower_Buttercup_A', .35),
    ('flower_purple', 'Flower_Lavender_A', .35), ('pot', 'Pot_M', .5),
    ('chair', 'SunsetChair', .98),
]:
    model(key, 'b612', name, height=height)

model('king.king', 'king', 'king_body', 'king_robe', 'king_crown', 'king_scepter', 'king_skirt', 'king_skirt_trim', height=1.65)
model('king.throne', 'king', 'RC_P2_02_Throne_Mesh', height=2.1)
model('king.candelabrum', 'king', 'RC_Candelabrum_T3_Mesh', height=1.7)
model('king.door', 'king', 'RoyalCourt_DoorLeaf', size=[1.4, 2.4, .14])
model('king.banner', 'king', 'RC_RoyalBanner_Mesh', height=2.0)

model('vanity.vain', 'theater', 'performer_body', 'performer_hat', height=1.95, pose='standing')
model('vanity.mirror', 'theater', 'P02_.*', height=3)
model('vanity.theatre', 'theater', 'VT_.*', preserve=True)
model('bar.drinker', 'bar', 'drinker_body', height=1.4)
model('bar.table', 'bar', 'PlankTable_Drinker', size=[1.0, .75, .86])
model('bar.chair', 'bar', 'Chair_LOD0', height=.95)
model('bar.chair_high', 'bar', 'Chair_LOD0', height=.95)
model('bar.barrel', 'bar', 'Barrel_LOD0', height=.9)
model('bar.door', 'bar', 'Bar_DoorFront_Leaf', 'Bar_DoorFront_Iron', size=[1.3, 2.2, .12])
model('bar.door_back', 'bar', 'Bar_DoorBack_Leaf', 'Bar_DoorBack_Iron', size=[1, 2.2, .12])
model('bar.tavern', 'bar', 'Bar_(Exterior|Interior|Roof|Upstairs)_.*', preserve=True)

model('businessman.merchant', 'counting', 'merchant_body', 'merchant_pen', height=1.35, pose='seated')
model('businessman.telescope', 'counting', 'Prop_BrassTelescope', 'Prop_BrassTelescope_Tube', height=1.65)
model('businessman.observatory', 'counting', 'Shell_.*', preserve=True)
model('businessman.dome', 'counting', 'Dome_.*', preserve=True)
for key, name in [
    ('rug', 'SO_Int_StarRug'), ('desk', 'SO_Int_WritingDesk'), ('chair', 'SO_Int_DeskChair'),
    ('bookshelf', 'SO_Int_Bookshelf'), ('cabinet', 'SO_Int_ArchiveCabinet'),
    ('low_cabinet', 'SO_Int_LowCabinet'), ('side_desk', 'SO_Int_SideDesk'),
    ('table', 'SO_Int_SmallTable'), ('stool', 'SO_Int_Stool'),
    ('armillary', 'SO_Int_Armillary_M'), ('chart_north', 'SO_Int_Chart_North'),
    ('chart_moon', 'SO_Int_Chart_Moon'), ('chart_constellation', 'SO_Int_Chart_Constellation'),
    ('chart_zodiac', 'SO_Int_Chart_Zodiac'), ('chart_orbits', 'SO_Int_Chart_Orbits'),
    ('lantern', 'Prop_Lantern'), ('oil_lamp', 'SO_Int_OilLamp'),
    ('candles', 'SO_Int_CandleA'), ('sextant', 'SO_Int_Sextant'),
    ('scrolls', 'SO_Int_ScrollPair'), ('boxes', 'Prop_Crate_A'),
]:
    model('businessman.'+key, 'counting', name)
model('businessman.float_rock', 'counting', 'Asteroid_A', height=1.4)

model('lamp.lamplighter', 'lamp', 'lamplighter_body', 'lamplighter_wand', height=2.0, pose='standing')
model('lamp.street_lamp', 'lamp', 'Brass_.*', 'Column_.*', 'Foundation_.*', 'Glass_Pane_.*', 'Lantern_TripoIronHousing', 'Flame_Core', 'Flame_Outer', height=3.2, keep_root_rotation=True,
      aliases={'Flame_Core': 'part:flame', 'Glass_Pane_1_Door': 'part:glass'})
model('lamp.boulder', 'lamp', 'Lighter_Boulder_01', height=.8)
model('lamp.stone', 'lamp', 'Rock_01_LOD0', height=.15)
model('lamp.bush', 'b612', 'Shrub_Low_LOD0', height=.4)
model('lamp.tide_pool', 'lamp', 'Tide_pool', size=[1.6, .15, 1.6])
model('lamp.seat', 'lamp', 'Lighter_Boulder_02', height=.42)

model('geo.geographer', 'geo', 'geographer_body', 'geographer_coat_hem', height=1.43)
model('geo.table', 'geo', 'GEO_Prop_MapTable_TopFix', size=[1.6, .8, 1.1])
model('geo.door', 'geo', 'GEO_DoorLeaf_L', size=[1.3, 2.8, .14])
model('geo.cypress', 'geo', 'GEO_Veg_Cypress_A', height=4.8)
model('geo.library', 'geo', 'GEO_Shell_.*', 'GEO_Roof_.*', 'GEO_Skylight_Frame', 'GEO_Skylight_Glass', preserve=True)
model('tree', 'bar', 'Tree_RoundA_LOD0', height=4.6)
model('tree_oak', 'bar', 'Tree_RoundB_LOD0', height=4.6)
model('tree_pine', 'geo', 'GEO_Veg_Pine', height=4.6)

VALLEY = 'earth_scenes_desert_valley'
DUNES = 'earth_scenes_desert_dunes'
model('earth.fox', VALLEY, 'Fox_Body', size=[.35, .42, .75])
model('earth.snake', VALLEY, 'Snake_Body', height=.65)
model('earth.pilot', DUNES, 'pilot_body', height=1.75, pose='standing')
for key, name, height in [
    ('wheat', 'Wheat_A_LOD0', .9), ('apple_tree', 'AppleTree_LOD0', 4.6),
    ('fountain', 'Garden_Fountain_LOD0', 1.8), ('urn', 'Garden_Urn_LOD0', .7),
    ('topiary', 'Garden_Topiary_Ball_LOD0', 1.3),
]:
    model('earth.'+key, VALLEY, name, height=height)
for key, name, size in [
    ('fence', 'Fence_2m_LOD0', [2, 1, None]),
    ('rose_bed', 'RoseBush_LOD0', [1.9, .7, 1]),
    ('garden_wall', 'Garden_RoseWallCap_3m_LOD0', [2, 1.6, None]),
    ('hedge', 'Garden_Hedge_Box_2m_LOD0', [2, 1.1, None]),
    ('gate', 'Garden_IronGate_Double_Open_LOD0', [2.8, 2.1, None]),
    ('arch', 'Garden_BackArch_LOD0', [2.4, 2.8, None]),
    ('bridge', 'Footbridge_LOD0', [1.6, None, 3.4]),
    ('cottage', 'StoneCottage_LOD0', [3.2, None, 2.8]),
]:
    model('earth.'+key, VALLEY, name, size=size)
for key, name, size in [
    ('plane', 'PlaneWreck_LOD0_EngineSplit_v1', [9.4, None, 6.4]),
    ('loco', 'SteamLocomotive_LOD0', [None, None, 6.3]),
    ('carriage', 'PassengerCarriage_LOD0', [None, None, 6.5]),
    ('cabin', 'Station_Cabin_LOD0', [3.2, None, 2.4]),
    ('rail', 'Station_Track_10m_LOD0', [1.5, None, 2]),
    ('bedroll', 'Pilot_Bedroll_Open_LOD0', [.8, None, 2]),
]:
    model('earth.'+key, DUNES, name, size=size)
for key, name, height in [
    ('bench', 'Station_Bench_LOD0', .85), ('buffer', 'Station_BufferStop_LOD0', .9),
    ('wrench', 'Pilot_Wrench_LOD0', .3), ('signpost', 'Signpost_LOD0', 1.5),
    ('column', 'Ruin_Column_Tall_LOD0', 3.2), ('column_broken', 'Ruin_Column_Short_LOD0', 1.8),
    ('block', 'Ruin_Block_A_LOD0', .6),
]:
    model('earth.'+key, DUNES, name, height=height)

AUDIO = {
    'music.title': ('b612', 'B612_Music'),
    'music.map': ('b612', 'B612_Music'),
    'music.ending': ('earth', 'DesertDunes_Music'),
    'music.b612': ('b612', 'B612_Music'),
    'music.king': ('king', 'King_Music'),
    'music.vanity': ('theater', 'Theater_Music'),
    'music.bar': ('bar', 'Bar_Music'),
    'music.businessman': ('counting', 'StarObservatory_Music'),
    'music.lamp': ('lamp', 'Lamplighter_Music'),
    'music.geo': ('geo', 'Geography_Music'),
    'music.earth': ('earth', 'DesertValley_Music'),
    'music.earth_dunes': ('earth', 'DesertDunes_Music'),
    'music.earth_end': ('earth', 'DesertDunes_Music'),
    'sfx.step': ('b612', 'Footstep_Grass'),
    'sfx.step_stone': ('b612', 'Footstep_Gravel'),
    'amb.wind': ('b612', 'Amb_Meadow_Day_Loop'),
    'amb.night': ('b612', 'Amb_Meadow_Dusk_Loop'),
    'amb.lamp.sea': ('lamp', 'Sea_Waves_Loop'),
    'sfx.lamp.light': ('lamp', 'Lamp_Light'),
    'sfx.lamp.out': ('lamp', 'Lamp_PutOut'),
    'sfx.lamp.surf': ('lamp', 'Sea_Waves_Loop'),
    'sfx.king.creak': ('king', 'Door_Creak'),
    'sfx.king.latch': ('king', 'Door_Latch'),
    'sfx.businessman.latch': ('counting', 'Door_Latch'),
    'sfx.businessman.creak': ('counting', 'Door_Creak'),
    'sfx.businessman.settle': ('counting', 'Door_Settle'),
    'sfx.businessman.scratch': ('counting', 'Merchant_Writing_Loop'),
    'sfx.geo.pencil': ('geo', 'Pencil_Writing_Loop'),
    'sfx.bar.gulp': ('bar', 'Drink_Swig'),
    'sfx.bar.clink': ('bar', 'Bottle_Set'),
    'sfx.earth.train': ('earth', 'Train_Arrive'),
    'sfx.earth.creak': ('earth', 'Well_Pulley_Loop'),
}
for i in range(4):
    AUDIO[f'sfx.vanity.clap{i}'] = ('theater', 'Clap')
