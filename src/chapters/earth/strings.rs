//! Earth chapter text (our own wording; beat ids follow the original's line
//! constants S/R/F/L so the structure can be compared, never its text).

use crate::core::prelude::*;

pub const SNAKE: Speaker = Speaker::new("earth.sp.snake", Color::srgb(0.78, 0.66, 0.32));
pub const FLOWER: Speaker = Speaker::new("earth.sp.flower", Color::srgb(0.9, 0.7, 0.78));
pub const ECHO: Speaker = Speaker::new("earth.sp.echo", Color::srgb(0.62, 0.62, 0.7));
pub const ROSES: Speaker = Speaker::new("earth.sp.roses", Color::srgb(0.88, 0.32, 0.36));
pub const FOX: Speaker = Speaker::new("earth.sp.fox", Color::srgb(0.85, 0.5, 0.22));
pub const PILOT: Speaker = Speaker::new("earth.sp.pilot", Color::srgb(0.55, 0.42, 0.3));

pub const STRINGS: StringTable = &[
    // ---- map / card / loading
    ("earth.title", "地球", "The Earth"),
    ("earth.subtitle", "一颗很大的星球，却一个人也看不见", "A very large planet, and nobody in sight"),
    ("earth.line", "地理学家说：去看看地球吧。那上面的沙漠里，有一颗特别亮的星星。", "The geographer said: go and see the Earth. Over its desert shines one especially bright star."),
    ("earth.visited", "他就在那里。沙漠之所以美丽，是因为某处藏着一口井。", "He is there. The desert is lovely because somewhere it hides a well."),
    ("earth.loading", "一颗很大很大的星球……", "A very, very large planet…"),
    // ---- speakers
    ("earth.sp.snake", "蛇", "The Snake"),
    ("earth.sp.flower", "沙漠里的花", "The Desert Flower"),
    ("earth.sp.echo", "回声", "The Echo"),
    ("earth.sp.roses", "玫瑰们", "The Roses"),
    ("earth.sp.fox", "狐狸", "The Fox"),
    ("earth.sp.pilot", "飞行员", "The Aviator"),
    // ---- events & steps (goal list)
    ("earth.ev.where", "弄清自己在哪里", "Find out where you are"),
    ("earth.st.snake", "和石头上的蛇说说话", "Talk to the snake on the rock"),
    ("earth.ev.roses", "去玫瑰园看看", "Visit the rose garden"),
    ("earth.st.roses", "跟着路牌，翻过山口去玫瑰园", "Follow the signs over the pass to the rose garden"),
    ("earth.ev.fox", "去苹果树下找狐狸", "Find the fox under the apple tree"),
    ("earth.st.fox", "穿过花园后门，过桥去苹果树下", "Leave by the back arch and cross the bridge to the apple tree"),
    ("earth.ev.train", "启程回家", "Set out for home"),
    ("earth.st.train", "去车站，登上火车", "Go to the station and board the train"),
    ("earth.ev.wake", "叫醒飞行员", "Wake the aviator"),
    ("earth.st.wake", "叫醒飞机旁睡着的人", "Wake the man sleeping by the plane"),
    ("earth.ev.home", "回到你落下的地方", "Return to where you fell"),
    ("earth.st.home", "翻过大沙丘，去找那口井", "Cross the big dune and find the well"),
    ("earth.st.way", "沿着石路走到门洞", "Follow the stone path to the gateway"),
    ("earth.st.pilot", "和飞行员道别", "Say goodbye to the aviator"),
    ("earth.hidden", "听听沙漠里那朵小花的话，再在山口喊一声", "Hear the little desert flower, and call out from the pass"),
    // ---- targets (marker labels)
    ("earth.tg.snake", "蛇", "The snake"),
    ("earth.tg.garden", "玫瑰园", "Rose garden"),
    ("earth.tg.fox", "苹果树", "Apple tree"),
    ("earth.tg.station", "车站", "Station"),
    ("earth.tg.pilot", "飞行员", "The aviator"),
    ("earth.tg.well", "水井", "The well"),
    ("earth.tg.gate", "门洞", "The gateway"),
    // ---- verbs
    ("earth.verb.snake", "和蛇说话", "Talk to the snake"),
    ("earth.verb.talk", "说话", "Talk"),
    ("earth.verb.call", "呼喊", "Call out"),
    ("earth.verb.roses", "看看玫瑰", "Look at the roses"),
    ("earth.verb.board", "登上火车", "Board the train"),
    ("earth.verb.wake", "叫醒他", "Wake him"),
    ("earth.verb.winch", "转动辘轳", "Turn the winch"),
    // ---- V1 the snake (S1–S9)
    ("earth.s1", "我掉到哪颗星球上来了？", "Which planet have I fallen onto?"),
    ("earth.s2", "地球。这里是非洲。", "The Earth. This is Africa."),
    ("earth.s3", "你看，我的星星就在我们头顶上……可它离得好远啊。", "Look, my star is right above us… but it is so far away."),
    ("earth.s4", "它真美。你到这里来做什么？", "It is beautiful. What brings you here?"),
    ("earth.s5", "我和一朵花闹了点别扭。", "I had some trouble with a flower."),
    ("earth.s6", "人都在哪儿呢？沙漠里有点孤单。", "Where are the people? It is a little lonely in the desert."),
    ("earth.s7", "在人群里，也一样孤单。", "It is lonely among people, too."),
    ("earth.s8", "被我碰过的，我都会把他送回他来的地方去。", "Whatever I touch, I send back to the ground it came from."),
    ("earth.s9", "你这么柔弱，站在这颗花岗岩一样硬的地球上……哪天你太想念你的星星了，我可以帮你。", "You are so frail on this granite Earth… If one day you miss your planet too much, I can help."),
    // ---- V-a the little flower (L10–L11)
    ("earth.l10", "你好。你见过人吗？", "Hello. Have you seen any people?"),
    ("earth.l11", "人？风把他们吹来吹去。他们没有根，所以活得很辛苦。", "People? The wind blows them about. They have no roots, and that makes life hard for them."),
    // ---- V-b the echo (L12, L13, L15)
    ("earth.l12", "你好！做我的朋友吧，我好孤单……", "Hello! Be my friends, I am lonely…"),
    ("earth.l13", "你好……你好……我好孤单……", "Hello… hello… I am lonely…"),
    ("earth.l15", "……孤单……孤单……", "…lonely… lonely…"),
    // ---- V2 the rose garden (R1–R3)
    ("earth.r1", "你们是谁？", "Who are you?"),
    ("earth.r2", "我们是玫瑰。", "We are roses."),
    ("earth.r3", "我以为我有一朵世上独一无二的花……原来，她只是一朵普通的玫瑰。", "I thought I had the only flower in the world… and she is just an ordinary rose."),
    // ---- V3 the fox (F1–F12)
    ("earth.f1", "来和我玩吧，我很难过。", "Come and play with me. I am so sad."),
    ("earth.f2", "我不能和你玩，我还没有被驯服。", "I cannot play with you. I have not been tamed."),
    ("earth.f3", "“驯服”是什么意思？", "What does “tame” mean?"),
    ("earth.f4", "就是“建立联系”。现在，你对我来说只是千万个小男孩里的一个。", "It means to make ties. Right now, to me, you are just one little boy among a hundred thousand."),
    ("earth.f5", "可要是你驯服了我，你对我来说就是世界上唯一的了。", "But if you tame me, you will be the only one in the world for me."),
    ("earth.f6", "我有点明白了……有一朵花，我想，她驯服了我。", "I am beginning to understand… There is a flower. I think she has tamed me."),
    ("earth.f7", "再去想想那些玫瑰吧。你会明白，你的那朵是独一无二的。", "Think of those roses again. You will see that yours is unique."),
    ("earth.f8", "那些玫瑰很美，可她们是空的。我的那朵更重要，因为是我给她浇的水。", "The roses are lovely, but they are empty. Mine matters more, because she is the one I watered."),
    ("earth.f9", "这是我的秘密：只有用心才能看清。真正重要的东西，眼睛是看不见的。", "Here is my secret: one sees clearly only with the heart. What matters most is invisible to the eyes."),
    ("earth.f10", "是你为你的玫瑰花费的时间，让她变得那么重要。", "It is the time you gave your rose that makes her so important."),
    ("earth.f11", "你要永远对你驯服过的东西负责。", "You are responsible, forever, for what you have tamed."),
    ("earth.f12", "我要对我的玫瑰负责……", "I am responsible for my rose…"),
    // ---- railway page
    ("earth.page.chapter", "第八章 · 地球", "Chapter Eight · The Earth"),
    ("earth.page.title", "沙丘", "The Dunes"),
    ("earth.page.sub", "地球", "The Earth"),
    ("earth.page.line", "最后一段路了。他的玫瑰，就在头顶那颗星星上。", "The last stretch of the way. His rose is on the star overhead."),
    ("earth.page.from", "山谷", "The valley"),
    ("earth.page.to", "沙丘", "The dunes"),
    // ---- D2 wake and draw (L36, L73, L37, L38, L65)
    ("earth.l36", "请你……给我画一只羊吧！", "Please… draw me a sheep!"),
    ("earth.l73", "这只太老了。我想要一只能活很久的羊。", "This one is too old. I want a sheep that will live a long time."),
    ("earth.l37", "这是一只箱子。你要的羊就在里面。", "This is a box. The sheep you want is inside."),
    ("earth.l38", "这正是我想要的！", "That is exactly what I wanted!"),
    ("earth.l65", "你看……它睡着了。", "Look… he has fallen asleep."),
    // ---- D3 title card + crate talk (L39, L71, L74, L75, L49)
    ("earth.card8", "八天以后", "Eight days later"),
    ("earth.card8.sub", "小王子讲完了他一路的旅行。", "The little prince had told the story of his journey."),
    ("earth.l39", "你的回忆真美。可我的飞机还没修好。", "Your memories are lovely. But my plane is still not fixed."),
    ("earth.l71", "今晚就满一年了。我的星星会刚好在我落下的地方的正上方。", "Tonight it will be one year. My star will be right above the place where I fell."),
    ("earth.l74", "你要回到你落下的地方去？", "You are going back to where you fell?"),
    ("earth.l75", "嗯……", "Mm…"),
    ("earth.l49", "你会看到沙子上我的脚印是从哪里开始的。在那儿等我吧，今晚我会在那里。", "You will see where my footprints begin in the sand. Wait for me there. Tonight I will be there."),
    // ---- D4 the well (L69, L79)
    ("earth.l69", "沙漠之所以美丽，是因为它在某个地方藏着一口井。", "What makes the desert beautiful is that somewhere it hides a well."),
    ("earth.l79", "真正重要的东西，眼睛是看不见的。", "What matters most is invisible to the eyes."),
    // ---- D5 the doorway snake (L76)
    ("earth.l76", "今晚……如果你太想念你的星星，我可以帮你。", "Tonight… if you miss your star too much, I can help you."),
    // ---- D6 farewell (L72, L50, L77, L51, L52, L53, L57, L58)
    ("earth.l72", "啊！你在这儿……", "Ah! You are here…"),
    ("earth.l50", "我很高兴你找到了飞机缺的东西。你可以回家了。", "I am glad you found what your plane was missing. You can go home now."),
    ("earth.l77", "你怎么知道的？", "How did you know?"),
    ("earth.l51", "我今天也要回家了……要远得多……也难得多……", "I am going home today, too… much farther… and much harder…"),
    ("earth.l52", "小家伙，我还想再听到你笑。", "Little fellow, I want to hear you laugh again."),
    ("earth.l53", "夜里你看星星的时候，我会在其中一颗上笑着。所以对你来说，所有的星星都会笑。", "At night, when you look at the sky, I will be laughing on one of the stars. So for you, all the stars will laugh."),
    ("earth.l57", "我的花……我要对她负责。她那么柔弱，只有四根刺。", "My flower… I am responsible for her. She is so weak, with only four thorns."),
    ("earth.l58", "就是这样……就这些了……", "There… that is all…"),
    // ---- end card
    ("earth.end.q1", "是你为你的玫瑰花费的时间，", "It is the time you gave your rose"),
    ("earth.end.q2", "让她变得如此重要。", "that makes her so important."),
    ("earth.end.ded", "献给每一个还记得星星会笑的人", "For everyone who remembers that the stars can laugh"),
    ("earth.end.title", "小王子", "The Little Prince"),
    ("earth.end.fin", "完", "The End"),
];
