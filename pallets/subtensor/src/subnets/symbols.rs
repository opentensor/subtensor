use super::*;
use sp_std::collections::btree_set::BTreeSet;
use subtensor_runtime_common::NetUid;

// TODO: default symbol should be different from the root symbol?
pub static DEFAULT_SYMBOL: &[u8] = b"\xCE\xA4"; // TAO uppercase symbol

pub static SYMBOLS: [&[u8]; 439] = [
    // Greek Alphabet (Lowercase)
    DEFAULT_SYMBOL, // Τ (Upper case Tau, 0)
    b"\xCE\xB1",    // α (Alpha, 1)
    b"\xCE\xB2",    // β (Beta, 2)
    b"\xCE\xB3",    // γ (Gamma, 3)
    b"\xCE\xB4",    // δ (Delta, 4)
    b"\xCE\xB5",    // ε (Epsilon, 5)
    b"\xCE\xB6",    // ζ (Zeta, 6)
    b"\xCE\xB7",    // η (Eta, 7)
    b"\xCE\xB8",    // θ (Theta, 8)
    b"\xCE\xB9",    // ι (Iota, 9)
    b"\xCE\xBA",    // κ (Kappa, 10)
    b"\xCE\xBB",    // λ (Lambda, 11)
    b"\xCE\xBC",    // μ (Mu, 12)
    b"\xCE\xBD",    // ν (Nu, 13)
    b"\xCE\xBE",    // ξ (Xi, 14)
    b"\xCE\xBF",    // ο (Omicron, 15)
    b"\xCF\x80",    // π (Pi, 16)
    b"\xCF\x81",    // ρ (Rho, 17)
    b"\xCF\x83",    // σ (Sigma, 18)
    b"t",           // t (Tau, 19)
    b"\xCF\x85",    // υ (Upsilon, 20)
    b"\xCF\x86",    // φ (Phi, 21)
    b"\xCF\x87",    // χ (Chi, 22)
    b"\xCF\x88",    // ψ (Psi, 23)
    b"\xCF\x89",    // ω (Omega, 24)
    // Hebrew Alphabet (Including Final Forms)
    b"\xD7\x90", // א (Aleph, 25)
    b"\xD7\x91", // ב (Bet, 26)
    b"\xD7\x92", // ג (Gimel, 27)
    b"\xD7\x93", // ד (Dalet, 28)
    b"\xD7\x94", // ה (He, 29)
    b"\xD7\x95", // ו (Vav, 30)
    b"\xD7\x96", // ז (Zayin, 31)
    b"\xD7\x97", // ח (Het, 32)
    b"\xD7\x98", // ט (Tet, 33)
    b"\xD7\x99", // י (Yod, 34)
    b"\xD7\x9A", // ך (Final Kaf, 35)
    b"\xD7\x9B", // כ (Kaf, 36)
    b"\xD7\x9C", // ל (Lamed, 37)
    b"\xD7\x9D", // ם (Final Mem, 38)
    b"\xD7\x9E", // מ (Mem, 39)
    b"\xD7\x9F", // ן (Final Nun, 40)
    b"\xD7\xA0", // נ (Nun, 41)
    b"\xD7\xA1", // ס (Samekh, 42)
    b"\xD7\xA2", // ע (Ayin, 43)
    b"\xD7\xA3", // ף (Final Pe, 44)
    b"\xD7\xA4", // פ (Pe, 45)
    b"\xD7\xA5", // ץ (Final Tsadi, 46)
    b"\xD7\xA6", // צ (Tsadi, 47)
    b"\xD7\xA7", // ק (Qof, 48)
    b"\xD7\xA8", // ר (Resh, 49)
    b"\xD7\xA9", // ש (Shin, 50)
    b"\xD7\xAA", // ת (Tav, 51)
    // Arabic Alphabet
    b"\xD8\xA7", // ا (Alif, 52)
    b"\xD8\xA8", // ب (Ba, 53)
    b"\xD8\xAA", // ت (Ta, 54)
    b"\xD8\xAB", // ث (Tha, 55)
    b"\xD8\xAC", // ج (Jim, 56)
    b"\xD8\xAD", // ح (Ha, 57)
    b"\xD8\xAE", // خ (Kha, 58)
    b"\xD8\xAF", // د (Dal, 59)
    b"\xD8\xB0", // ذ (Dhal, 60)
    b"\xD8\xB1", // ر (Ra, 61)
    b"\xD8\xB2", // ز (Zay, 62)
    b"\xD8\xB3", // س (Sin, 63)
    b"\xD8\xB4", // ش (Shin, 64)
    b"\xD8\xB5", // ص (Sad, 65)
    b"\xD8\xB6", // ض (Dad, 66)
    b"\xD8\xB7", // ط (Ta, 67)
    b"\xD8\xB8", // ظ (Dha, 68)
    b"\xD8\xB9", // ع (Ain, 69)
    b"\xD8\xBA", // غ (Ghayn, 70)
    b"\xD9\x81", // ف (Fa, 71)
    b"\xD9\x82", // ق (Qaf, 72)
    b"\xD9\x83", // ك (Kaf, 73)
    b"\xD9\x84", // ل (Lam, 74)
    b"\xD9\x85", // م (Mim, 75)
    b"\xD9\x86", // ن (Nun, 76)
    b"\xD9\x87", // ه (Ha, 77)
    b"\xD9\x88", // و (Waw, 78)
    b"\xD9\x8A", // ي (Ya, 79)
    b"\xD9\x89", // ى (Alef Maksura, 80)
    // Runic Alphabet
    b"\xE1\x9A\xA0", // ᚠ (Fehu, wealth, 81)
    b"\xE1\x9A\xA2", // ᚢ (Uruz, strength, 82)
    b"\xE1\x9A\xA6", // ᚦ (Thurisaz, giant, 83)
    b"\xE1\x9A\xA8", // ᚨ (Ansuz, god, 84)
    b"\xE1\x9A\xB1", // ᚱ (Raidho, ride, 85)
    b"\xE1\x9A\xB3", // ᚳ (Kaunan, ulcer, 86)
    b"\xD0\xAB",     // Ы (Cyrillic Yeru, 87)
    b"\xE1\x9B\x89", // ᛉ (Algiz, protection, 88)
    b"\xE1\x9B\x92", // ᛒ (Berkanan, birch, 89)
    // Ogham Alphabet
    b"\xE1\x9A\x80", //   (Space, 90)
    b"\xE1\x9A\x81", // ᚁ (Beith, birch, 91)
    b"\xE1\x9A\x82", // ᚂ (Luis, rowan, 92)
    b"\xE1\x9A\x83", // ᚃ (Fearn, alder, 93)
    b"\xE1\x9A\x84", // ᚄ (Sail, willow, 94)
    b"\xE1\x9A\x85", // ᚅ (Nion, ash, 95)
    b"\xE1\x9A\x9B", // ᚛ (Forfeda, 96)
    // Georgian Alphabet (Mkhedruli)
    b"\xE1\x83\x90", // ა (Ani, 97)
    b"\xE1\x83\x91", // ბ (Bani, 98)
    b"\xE1\x83\x92", // გ (Gani, 99)
    b"\xE1\x83\x93", // დ (Doni, 100)
    b"\xE1\x83\x94", // ე (Eni, 101)
    b"\xE1\x83\x95", // ვ (Vini, 102)
    // Armenian Alphabet
    b"\xD4\xB1", // Ա (Ayp, 103)
    b"\xD4\xB2", // Բ (Ben, 104)
    b"\xD4\xB3", // Գ (Gim, 105)
    b"\xD4\xB4", // Դ (Da, 106)
    b"\xD4\xB5", // Ե (Ech, 107)
    b"\xD4\xB6", // Զ (Za, 108)
    b"\xD5\x9E", // ՞ (Question mark, 109)
    // Cyrillic Alphabet
    b"\xD0\x80", // Ѐ (Ie with grave, 110)
    b"\xD0\x81", // Ё (Io, 111)
    b"\xD0\x82", // Ђ (Dje, 112)
    b"\xD0\x83", // Ѓ (Gje, 113)
    b"\xD0\x84", // Є (Ukrainian Ie, 114)
    b"\xD0\x85", // Ѕ (Dze, 115)
    b"\xD1\x8A", // ъ (Hard sign, 116)
    // Coptic Alphabet
    b"\xE2\xB2\x80", // Ⲁ (Alfa, 117)
    b"\xE2\xB2\x81", // ⲁ (Small Alfa, 118)
    b"\xE2\xB2\x82", // Ⲃ (Vida, 119)
    b"\xE2\xB2\x83", // ⲃ (Small Vida, 120)
    b"\xE2\xB2\x84", // Ⲅ (Gamma, 121)
    b"\xE2\xB2\x85", // ⲅ (Small Gamma, 122)
    // Brahmi Script
    b"\xF0\x91\x80\x80", // 𑀀 (A, 123)
    b"\xF0\x91\x80\x81", // 𑀁 (Aa, 124)
    b"\xF0\x91\x80\x82", // 𑀂 (I, 125)
    b"\xF0\x91\x80\x83", // 𑀃 (Ii, 126)
    b"\xF0\x91\x80\x85", // 𑀅 (U, 127)
    // End of Sinhala Alphabet
    b"\xE0\xB6\xB1", // න (La, 128)
    b"\xE0\xB6\xB3", // ඳ (Va, 129)
    b"\xE0\xB6\xB4", // ප (Sha, 130)
    b"\xE0\xB6\xB5", // ඵ (Ssa, 131)
    b"\xE0\xB6\xB6", // බ (Sa, 132)
    b"\xE0\xB6\xB7", // භ (Ha, 133)
    // Glagolitic Alphabet
    b"\xE2\xB0\x80", // Ⰰ (Az, 134)
    b"\xE2\xB0\x81", // Ⰱ (Buky, 135)
    b"\xE2\xB0\x82", // Ⰲ (Vede, 136)
    b"\xE2\xB0\x83", // Ⰳ (Glagoli, 137)
    b"\xE2\xB0\x84", // Ⰴ (Dobro, 138)
    b"\xE2\xB0\x85", // Ⰵ (Yest, 139)
    b"\xE2\xB0\x86", // Ⰶ (Zhivete, 140)
    b"\xE2\xB0\x87", // Ⰷ (Zemlja, 141)
    b"\xE2\xB0\x88", // Ⰸ (Izhe, 142)
    b"\xE2\xB0\x89", // Ⰹ (Initial Izhe, 143)
    b"\xE2\xB0\x8A", // Ⰺ (I, 144)
    b"\xE2\xB0\x8B", // Ⰻ (Djerv, 145)
    b"\xE2\xB0\x8C", // Ⰼ (Kako, 146)
    b"\xE2\xB0\x8D", // Ⰽ (Ljudije, 147)
    b"\xE2\xB0\x8E", // Ⰾ (Myse, 148)
    b"\xE2\xB0\x8F", // Ⰿ (Nash, 149)
    b"\xE2\xB0\x90", // Ⱀ (On, 150)
    b"\xE2\xB0\x91", // Ⱁ (Pokoj, 151)
    b"\xE2\xB0\x92", // Ⱂ (Rtsy, 152)
    b"\xE2\xB0\x93", // Ⱃ (Slovo, 153)
    b"\xE2\xB0\x94", // Ⱄ (Tvrido, 154)
    b"\xE2\xB0\x95", // Ⱅ (Uku, 155)
    b"\xE2\xB0\x96", // Ⱆ (Fert, 156)
    b"\xE2\xB0\x97", // Ⱇ (Xrivi, 157)
    b"\xE2\xB0\x98", // Ⱈ (Ot, 158)
    b"\xE2\xB0\x99", // Ⱉ (Cy, 159)
    b"\xE2\xB0\x9A", // Ⱊ (Shcha, 160)
    b"\xE2\xB0\x9B", // Ⱋ (Er, 161)
    b"\xE2\xB0\x9C", // Ⱌ (Yeru, 162)
    b"\xE2\xB0\x9D", // Ⱍ (Small Yer, 163)
    b"\xE2\xB0\x9E", // Ⱎ (Yo, 164)
    b"\xE2\xB0\x9F", // Ⱏ (Yu, 165)
    b"\xE2\xB0\xA0", // Ⱐ (Ja, 166)
    // Thai Alphabet
    b"\xE0\xB8\x81", // ก (Ko Kai, 167)
    b"\xE0\xB8\x82", // ข (Kho Khai, 168)
    b"\xE0\xB8\x83", // ฃ (Kho Khuat, 169)
    b"\xE0\xB8\x84", // ค (Kho Khon, 170)
    b"\xE0\xB8\x85", // ฅ (Kho Rakhang, 171)
    b"\xE0\xB8\x86", // ฆ (Kho Khwai, 172)
    b"\xE0\xB8\x87", // ง (Ngo Ngu, 173)
    b"\xE0\xB8\x88", // จ (Cho Chan, 174)
    b"\xE0\xB8\x89", // ฉ (Cho Ching, 175)
    b"\xE0\xB8\x8A", // ช (Cho Chang, 176)
    b"\xE0\xB8\x8B", // ซ (So So, 177)
    b"\xE0\xB8\x8C", // ฌ (Cho Choe, 178)
    b"\xE0\xB8\x8D", // ญ (Yo Ying, 179)
    b"\xE0\xB8\x8E", // ฎ (Do Chada, 180)
    b"\xE0\xB8\x8F", // ฏ (To Patak, 181)
    b"\xE0\xB8\x90", // ฐ (Tho Than, 182)
    b"\xE0\xB8\x91", // ฑ (Tho Nangmontho, 183)
    b"\xE0\xB8\x92", // ฒ (Tho Phuthao, 184)
    b"\xE0\xB8\x93", // ณ (No Nen, 185)
    b"\xE0\xB8\x94", // ด (Do Dek, 186)
    b"\xE0\xB8\x95", // ต (To Tao, 187)
    b"\xE0\xB8\x96", // ถ (Tho Thung, 188)
    b"\xE0\xB8\x97", // ท (Tho Thahan, 189)
    b"\xE0\xB8\x98", // ธ (Tho Thong, 190)
    b"\xE0\xB8\x99", // น (No Nu, 191)
    b"\xE0\xB8\x9A", // บ (Bo Baimai, 192)
    b"\xE0\xB8\x9B", // ป (Po Pla, 193)
    b"\xE0\xB8\x9C", // ผ (Pho Phung, 194)
    b"\xE0\xB8\x9D", // ฝ (Fo Fa, 195)
    b"\xE0\xB8\x9E", // พ (Pho Phan, 196)
    b"\xE0\xB8\x9F", // ฟ (Fo Fan, 197)
    b"\xE0\xB8\xA0", // ภ (Pho Samphao, 198)
    b"\xE0\xB8\xA1", // ม (Mo Ma, 199)
    b"\xE0\xB8\xA2", // ย (Yo Yak, 200)
    b"\xE0\xB8\xA3", // ร (Ro Rua, 201)
    b"\xE0\xB8\xA5", // ล (Lo Ling, 202)
    b"\xE0\xB8\xA7", // ว (Wo Waen, 203)
    b"\xE0\xB8\xA8", // ศ (So Sala, 204)
    b"\xE0\xB8\xA9", // ษ (So Rusi, 205)
    b"\xE0\xB8\xAA", // ส (So Sua, 206)
    b"\xE0\xB8\xAB", // ห (Ho Hip, 207)
    b"\xE0\xB8\xAC", // ฬ (Lo Chula, 208)
    b"\xE0\xB8\xAD", // อ (O Ang, 209)
    b"\xE0\xB8\xAE", // ฮ (Ho Nokhuk, 210)
    // Hangul Alphabet (Korean)
    b"\xE3\x84\xB1", // ㄱ (Giyeok, 211)
    b"\xE3\x84\xB4", // ㄴ (Nieun, 212)
    b"\xE3\x84\xB7", // ㄷ (Digeut, 213)
    b"\xE3\x84\xB9", // ㄹ (Rieul, 214)
    b"\xE3\x85\x81", // ㅁ (Mieum, 215)
    b"\xE3\x85\x82", // ㅂ (Bieup, 216)
    b"\xE3\x85\x85", // ㅅ (Siot, 217)
    b"\xE3\x85\x87", // ㅇ (Ieung, 218)
    b"\xE3\x85\x88", // ㅈ (Jieut, 219)
    b"\xE3\x85\x8A", // ㅊ (Chieut, 220)
    b"\xE3\x85\x8B", // ㅋ (Kieuk, 221)
    b"\xE3\x85\x8C", // ㅌ (Tieut, 222)
    b"\xE3\x85\x8D", // ㅍ (Pieup, 223)
    b"\xE3\x85\x8E", // ㅎ (Hieut, 224)
    // Hangul Vowels
    b"\xE3\x85\x8F", // ㅏ (A, 225)
    b"\xE3\x85\x90", // ㅐ (Ae, 226)
    b"\xE3\x85\x91", // ㅑ (Ya, 227)
    b"\xE3\x85\x92", // ㅒ (Yae, 228)
    b"\xE3\x85\x93", // ㅓ (Eo, 229)
    b"\xE3\x85\x94", // ㅔ (E, 230)
    b"\xE3\x85\x95", // ㅕ (Yeo, 231)
    b"\xE3\x85\x96", // ㅖ (Ye, 232)
    b"\xE3\x85\x97", // ㅗ (O, 233)
    b"\xE3\x85\x98", // ㅘ (Wa, 234)
    b"\xE3\x85\x99", // ㅙ (Wae, 235)
    b"\xE3\x85\x9A", // ㅚ (Oe, 236)
    b"\xE3\x85\x9B", // ㅛ (Yo, 237)
    b"\xE3\x85\x9C", // ㅜ (U, 238)
    b"\xE3\x85\x9D", // ㅝ (Weo, 239)
    b"\xE3\x85\x9E", // ㅞ (We, 240)
    b"\xE3\x85\x9F", // ㅟ (Wi, 241)
    b"\xE3\x85\xA0", // ㅠ (Yu, 242)
    b"\xE3\x85\xA1", // ㅡ (Eu, 243)
    b"\xE3\x85\xA2", // ㅢ (Ui, 244)
    b"\xE3\x85\xA3", // ㅣ (I, 245)
    // Ethiopic Alphabet
    b"\xE1\x8A\xA0", // አ (Glottal A, 246)
    b"\xE1\x8A\xA1", // ኡ (Glottal U, 247)
    b"\xE1\x8A\xA2", // ኢ (Glottal I, 248)
    b"\xE1\x8A\xA3", // ኣ (Glottal Aa, 249)
    b"\xE1\x8A\xA4", // ኤ (Glottal E, 250)
    b"\xE1\x8A\xA5", // እ (Glottal Ie, 251)
    b"\xE1\x8A\xA6", // ኦ (Glottal O, 252)
    b"\xE1\x8A\xA7", // ኧ (Glottal Wa, 253)
    b"\xE1\x8B\x88", // ወ (Wa, 254)
    b"\xE1\x8B\x89", // ዉ (Wu, 255)
    b"\xE1\x8B\x8A", // ዊ (Wi, 256)
    b"\xE1\x8B\x8B", // ዋ (Waa, 257)
    b"\xE1\x8B\x8C", // ዌ (We, 258)
    b"\xE1\x8B\x8D", // ው (Wye, 259)
    b"\xE1\x8B\x8E", // ዎ (Wo, 260)
    b"\xE1\x8A\xB0", // ኰ (Ko, 261)
    b"\xE1\x8A\xB1", // ኱ (Ku, 262)
    b"\xE1\x8A\xB2", // ኲ (Ki, 263)
    b"\xE1\x8A\xB3", // ኳ (Kua, 264)
    b"\xE1\x8A\xB4", // ኴ (Ke, 265)
    b"\xE1\x8A\xB5", // ኵ (Kwe, 266)
    b"\xE1\x8A\xB6", // ኶ (Ko, 267)
    b"\xE1\x8C\x90", // ጐ (Go, 268)
    b"\xE1\x8C\x91", // ጑ (Gu, 269)
    b"\xE1\x8C\x92", // ጒ (Gi, 270)
    b"\xE1\x88\x98", // መ (Gua, 271)
    b"\xE1\x8C\x94", // ጔ (Ge, 272)
    b"\xE1\x8C\x95", // ጕ (Gwe, 273)
    b"\xE1\x8C\x96", // ጖ (Go, 274)
    // Devanagari Alphabet
    b"\xE0\xA4\x85", // अ (A, 275)
    b"\xE0\xA4\x86", // आ (Aa, 276)
    b"\xE0\xA4\x87", // इ (I, 277)
    b"\xE0\xA4\x88", // ई (Ii, 278)
    b"\xE0\xA4\x89", // उ (U, 279)
    b"\xE0\xA4\x8A", // ऊ (Uu, 280)
    b"\xE0\xA4\x8B", // ऋ (R, 281)
    b"\xE0\xA4\x8F", // ए (E, 282)
    b"\xE0\xA4\x90", // ऐ (Ai, 283)
    b"\xE0\xA4\x93", // ओ (O, 284)
    b"\xE0\xA4\x94", // औ (Au, 285)
    b"\xE0\xA4\x95", // क (Ka, 286)
    b"\xE0\xA4\x96", // ख (Kha, 287)
    b"\xE0\xA4\x97", // ग (Ga, 288)
    b"\xE0\xA4\x98", // घ (Gha, 289)
    b"\xE0\xA4\x99", // ङ (Nga, 290)
    b"\xE0\xA4\x9A", // च (Cha, 291)
    b"\xE0\xA4\x9B", // छ (Chha, 292)
    b"\xE0\xA4\x9C", // ज (Ja, 293)
    b"\xE0\xA4\x9D", // झ (Jha, 294)
    b"\xE0\xA4\x9E", // ञ (Nya, 295)
    b"\xE0\xA4\x9F", // ट (Ta, 296)
    b"\xE0\xA4\xA0", // ठ (Tha, 297)
    b"\xE0\xA4\xA1", // ड (Da, 298)
    b"\xE0\xA4\xA2", // ढ (Dha, 299)
    b"\xE0\xA4\xA3", // ण (Na, 300)
    b"\xE0\xA4\xA4", // त (Ta, 301)
    b"\xE0\xA4\xA5", // थ (Tha, 302)
    b"\xE0\xA4\xA6", // द (Da, 303)
    b"\xE0\xA4\xA7", // ध (Dha, 304)
    b"\xE0\xA4\xA8", // न (Na, 305)
    b"\xE0\xA4\xAA", // प (Pa, 306)
    b"\xE0\xA4\xAB", // फ (Pha, 307)
    b"\xE0\xA4\xAC", // ब (Ba, 308)
    b"\xE0\xA4\xAD", // भ (Bha, 309)
    b"\xE0\xA4\xAE", // म (Ma, 310)
    b"\xE0\xA4\xAF", // य (Ya, 311)
    b"\xE0\xA4\xB0", // र (Ra, 312)
    b"\xE0\xA4\xB2", // ल (La, 313)
    b"\xE0\xA4\xB5", // व (Va, 314)
    b"\xE0\xA4\xB6", // श (Sha, 315)
    b"\xE0\xA4\xB7", // ष (Ssa, 316)
    b"\xE0\xA4\xB8", // स (Sa, 317)
    b"\xE0\xA4\xB9", // ह (Ha, 318)
    // Katakana Alphabet
    b"\xE3\x82\xA2", // ア (A, 319)
    b"\xE3\x82\xA4", // イ (I, 320)
    b"\xE3\x82\xA6", // ウ (U, 321)
    b"\xE3\x82\xA8", // エ (E, 322)
    b"\xE3\x82\xAA", // オ (O, 323)
    b"\xE3\x82\xAB", // カ (Ka, 324)
    b"\xE3\x82\xAD", // キ (Ki, 325)
    b"\xE3\x82\xAF", // ク (Ku, 326)
    b"\xE3\x82\xB1", // ケ (Ke, 327)
    b"\xE3\x82\xB3", // コ (Ko, 328)
    b"\xE3\x82\xB5", // サ (Sa, 329)
    b"\xE3\x82\xB7", // シ (Shi, 330)
    b"\xE3\x82\xB9", // ス (Su, 331)
    b"\xE3\x82\xBB", // セ (Se, 332)
    b"\xE3\x82\xBD", // ソ (So, 333)
    b"\xE3\x82\xBF", // タ (Ta, 334)
    b"\xE3\x83\x81", // チ (Chi, 335)
    b"\xE3\x83\x84", // ツ (Tsu, 336)
    b"\xE3\x83\x86", // テ (Te, 337)
    b"\xE3\x83\x88", // ト (To, 338)
    b"\xE3\x83\x8A", // ナ (Na, 339)
    b"\xE3\x83\x8B", // ニ (Ni, 340)
    b"\xE3\x83\x8C", // ヌ (Nu, 341)
    b"\xE3\x83\x8D", // ネ (Ne, 342)
    b"\xE3\x83\x8E", // ノ (No, 343)
    b"\xE3\x83\x8F", // ハ (Ha, 344)
    b"\xE3\x83\x92", // ヒ (Hi, 345)
    b"\xE3\x83\x95", // フ (Fu, 346)
    b"\xE3\x83\x98", // ヘ (He, 347)
    b"\xE3\x83\x9B", // ホ (Ho, 348)
    b"\xE3\x83\x9E", // マ (Ma, 349)
    b"\xE3\x83\x9F", // ミ (Mi, 350)
    b"\xE3\x83\xA0", // ム (Mu, 351)
    b"\xE3\x83\xA1", // メ (Me, 352)
    b"\xE3\x83\xA2", // モ (Mo, 353)
    b"\xE3\x83\xA4", // ヤ (Ya, 354)
    b"\xE3\x83\xA6", // ユ (Yu, 355)
    b"\xE3\x83\xA8", // ヨ (Yo, 356)
    b"\xE3\x83\xA9", // ラ (Ra, 357)
    b"\xE3\x83\xAA", // リ (Ri, 358)
    b"\xE3\x83\xAB", // ル (Ru, 359)
    b"\xE3\x83\xAC", // レ (Re, 360)
    b"\xE3\x83\xAD", // ロ (Ro, 361)
    b"\xE3\x83\xAF", // ワ (Wa, 362)
    b"\xE3\x83\xB2", // ヲ (Wo, 363)
    b"\xE3\x83\xB3", // ン (N, 364)
    // Tifinagh Alphabet
    b"\xE2\xB4\xB0", // ⴰ (Ya, 365)
    b"\xE2\xB4\xB1", // ⴱ (Yab, 366)
    b"\xE2\xB4\xB2", // ⴲ (Yabh, 367)
    b"\xE2\xB4\xB3", // ⴳ (Yag, 368)
    b"\xE2\xB4\xB4", // ⴴ (Yagh, 369)
    b"\xE2\xB4\xB5", // ⴵ (Yaj, 370)
    b"\xE2\xB4\xB6", // ⴶ (Yach, 371)
    b"\xE2\xB4\xB7", // ⴷ (Yad, 372)
    b"\xE2\xB4\xB8", // ⴸ (Yadh, 373)
    b"\xE2\xB4\xB9", // ⴹ (Yadh, emphatic, 374)
    b"\xE2\xB4\xBA", // ⴺ (Yaz, 375)
    b"\xE2\xB4\xBB", // ⴻ (Yazh, 376)
    b"\xE2\xB4\xBC", // ⴼ (Yaf, 377)
    b"\xE2\xB4\xBD", // ⴽ (Yak, 378)
    b"\xE2\xB4\xBE", // ⴾ (Yak, variant, 379)
    b"\xE2\xB4\xBF", // ⴿ (Yaq, 380)
    b"\xE2\xB5\x80", // ⵀ (Yah, 381)
    b"\xE2\xB5\x81", // ⵁ (Yahh, 382)
    b"\xE2\xB5\x82", // ⵂ (Yahl, 383)
    b"\xE2\xB5\x83", // ⵃ (Yahm, 384)
    b"\xE2\xB5\x84", // ⵄ (Yayn, 385)
    b"\xE2\xB5\x85", // ⵅ (Yakh, 386)
    b"\xE2\xB5\x86", // ⵆ (Yakl, 387)
    b"\xE2\xB5\x87", // ⵇ (Yahq, 388)
    b"\xE2\xB5\x88", // ⵈ (Yash, 389)
    b"\xE2\xB5\x89", // ⵉ (Yi, 390)
    b"\xE2\xB5\x8A", // ⵊ (Yij, 391)
    b"\xE2\xB5\x8B", // ⵋ (Yizh, 392)
    b"\xE2\xB5\x8C", // ⵌ (Yink, 393)
    b"\xE2\xB5\x8D", // ⵍ (Yal, 394)
    b"\xE2\xB5\x8E", // ⵎ (Yam, 395)
    b"\xE2\xB5\x8F", // ⵏ (Yan, 396)
    b"\xE2\xB5\x90", // ⵐ (Yang, 397)
    b"\xE2\xB5\x91", // ⵑ (Yany, 398)
    b"\xE2\xB5\x92", // ⵒ (Yap, 399)
    b"\xE2\xB5\x93", // ⵓ (Yu, 400)
    // Sinhala Alphabet
    b"\xE0\xB6\x85", // අ (A, 401)
    b"\xE0\xB6\x86", // ආ (Aa, 402)
    b"\xE0\xB6\x89", // ඉ (I, 403)
    b"\xE0\xB6\x8A", // ඊ (Ii, 404)
    b"\xE0\xB6\x8B", // උ (U, 405)
    b"\xE0\xB6\x8C", // ඌ (Uu, 406)
    b"\xE0\xB6\x8D", // ඍ (R, 407)
    b"\xE0\xB6\x8E", // ඎ (Rr, 408)
    b"\xE0\xB6\x8F", // ඏ (L, 409)
    b"\xE0\xB6\x90", // ඐ (Ll, 410)
    b"\xE0\xB6\x91", // එ (E, 411)
    b"\xE0\xB6\x92", // ඒ (Ee, 412)
    b"\xE0\xB6\x93", // ඓ (Ai, 413)
    b"\xE0\xB6\x94", // ඔ (O, 414)
    b"\xE0\xB6\x95", // ඕ (Oo, 415)
    b"\xE0\xB6\x96", // ඖ (Au, 416)
    b"\xE0\xB6\x9A", // ක (Ka, 417)
    b"\xE0\xB6\x9B", // ඛ (Kha, 418)
    b"\xE0\xB6\x9C", // ග (Ga, 419)
    b"\xE0\xB6\x9D", // ඝ (Gha, 420)
    b"\xE0\xB6\x9E", // ඞ (Nga, 421)
    b"\xE0\xB6\xA0", // ච (Cha, 422)
    b"\xE0\xB6\xA1", // ඡ (Chha, 423)
    b"\xE0\xB6\xA2", // ජ (Ja, 424)
    b"\xE0\xB6\xA3", // ඣ (Jha, 425)
    b"\xE0\xB6\xA4", // ඤ (Nya, 426)
    b"\xE0\xB6\xA7", // ට (Ta, 427)
    b"\xE0\xB6\xA5", // ඥ (Tha, 428)
    b"\xE0\xB6\xA6", // ඦ (Da, 429)
    b"\xE0\xB6\xA9", // ඩ (Dha, 430)
    b"\xE0\xB6\xA8", // ඨ (Na, 431)
    b"\xE0\xB6\xAA", // ඪ (Pa, 432)
    b"\xE0\xB6\xAB", // ණ (Pha, 433)
    b"\xE0\xB6\xAC", // ඬ (Ba, 434)
    b"\xE0\xB6\xAD", // ත (Bha, 435)
    b"\xE0\xB6\xAE", // ථ (Ma, 436)
    b"\xE0\xB6\xAF", // ද (Ya, 437)
    b"\xE0\xB6\xB0", // ධ (Ra, 438)
];

pub static NAMES: [&[u8]; 439] = [
    b"root",              // Τ (Upper case Tau)
    b"apex",              // α (Alpha)
    b"omron",             // β (Beta)
    b"templar",           // γ (Gamma)
    b"targon",            // δ (Delta)
    b"kaito",             // ε (Epsilon)
    b"infinite",          // ζ (Zeta)
    b"subvortex",         // η (Eta)
    b"ptn",               // θ (Theta)
    b"pretrain",          // ι (Iota)
    b"sturdy",            // κ (Kappa)
    b"dippy",             // λ (Lambda)
    b"horde",             // μ (Mu)
    b"dataverse",         // ν (Nu)
    b"palaidn",           // ξ (Xi)
    b"deval",             // ο (Omicron)
    b"bitads",            // π (Pi)
    b"3gen",              // ρ (Rho)
    b"cortex",            // σ (Sigma)
    b"inference",         // t (Tau)
    b"bitagent",          // υ (Upsilon)
    b"any-any",           // φ (Phi)
    b"meta",              // χ (Chi)
    b"social",            // ψ (Psi)
    b"omega",             // ω (Omega)
    b"protein",           // א (Aleph)
    b"alchemy",           // ב (Bet)
    b"compute",           // ג (Gimel)
    b"oracle",            // ד (Dalet)
    b"coldint",           // ה (He)
    b"bet",               // ו (Vav)
    b"naschain",          // ז (Zayin)
    b"itsai",             // ח (Het)
    b"ready",             // ט (Tet)
    b"mind",              // י (Yod)
    b"logic",             // ך (Final Kaf)
    b"automata",          // כ (Kaf)
    b"tuning",            // ל (Lamed)
    b"distributed",       // ם (Final Mem)
    b"edge",              // מ (Mem)
    b"chunk",             // ן (Final Nun)
    b"sportsensor",       // נ (Nun)
    b"masa",              // ס (Samekh)
    b"graphite",          // ע (Ayin)
    b"score",             // ף (Final Pe)
    b"gen42",             // פ (Pe)
    b"neural",            // ץ (Final Tsadi)
    b"condense",          // צ (Tsadi)
    b"nextplace",         // ק (Qof)
    b"automl",            // ר (Resh)
    b"audio",             // ש (Shin)
    b"celium",            // ת (Tav)
    b"dojo",              // ا (Alif)
    b"frontier",          // ب (Ba)
    b"safescan",          // ت (Ta)
    b"unknown",           // ث (Tha)
    b"gradients",         // ج (Jim)
    b"gaia",              // ح (Ha)
    b"dippy-speach",      // خ (Kha)
    b"agent-arena",       // د (Dal)
    b"unknown",           // ذ (Dhal)
    b"red team",          // ر (Ra)
    b"agentao",           // ز (Zay)
    b"lean-in",           // س (Sin)
    b"chutes",            // ش (Shin)
    b"sad",
    b"dad",
    b"ta",
    b"dha",
    b"ain",
    b"ghayn",
    b"fa",
    b"qaf",
    b"kaf",
    b"lam",
    b"mim",
    b"nun",
    b"ha",
    b"waw",
    b"ya",
    b"alef",
    b"fehu",
    b"uruz",
    b"thurisaz",
    b"ansuz",
    b"raidho",
    b"kaunan",
    b"cyr_yeru",
    b"algiz",
    b"berkanan",
    b"ogham",
    b"beith",
    b"luis",
    b"fearn",
    b"sail",
    b"nion",
    b"forfeda",
    b"ani",
    b"bani",
    b"gani",
    b"doni",
    b"eni",
    b"vini",
    b"ayp",
    b"ben",
    b"gim",
    b"da",
    b"ech",
    b"za",
    b"armeni",
    b"grave",
    b"io",
    b"dje",
    b"gje",
    b"ie",
    b"dze",
    b"hard_sign",
    b"alfa",
    b"alfas",
    b"vida",              // Ⲃ (Vida, 119)
    b"vida_small",        // ⲃ (Small Vida, 120)
    b"gamma",             // Ⲅ (Gamma, 121)
    b"gamma_small",       // ⲅ (Small Gamma, 122)
    b"brahmi_a",          // 𑀀 (A, 123)
    b"brahmi_aa",         // 𑀁 (Aa, 124)
    b"brahmi_i",          // 𑀂 (I, 125)
    b"brahmi_ii",         // 𑀃 (Ii, 126)
    b"brahmi_u",          // 𑀅 (U, 127)
    b"la",
    b"va",
    b"sha",
    b"ssa",
    b"sa",
    b"ha",
    b"glagolitic_az",     // Ⰰ (Az, 134)
    b"glagolitic_buky",   // Ⰱ (Buky, 135)
    b"glagolitic_vede",   // Ⰲ (Vede, 136)
    b"glagolitic_glagoli",// Ⰳ (Glagoli, 137)
    b"glagolitic_dobro",  // Ⰴ (Dobro, 138)
    b"glagolitic_yest",   // Ⰵ (Yest, 139)
    b"glagolitic_zhivete",// Ⰶ (Zhivete, 140)
    b"glagolitic_zemlja", // Ⰷ (Zemlja, 141)
    b"glagolitic_izhe",   // Ⰸ (Izhe, 142)
    b"glagolitic_initial_izhe",// Ⰹ (Initial Izhe, 143)
    b"glagolitic_i",      // Ⰺ (I, 144)
    b"glagolitic_djerv",  // Ⰻ (Djerv, 145)
    b"glagolitic_kako",   // Ⰼ (Kako, 146)
    b"glagolitic_ljudije",// Ⰽ (Ljudije, 147)
    b"glagolitic_myse",   // Ⰾ (Myse, 148)
    b"glagolitic_nash",   // Ⰿ (Nash, 149)
    b"glagolitic_on",     // Ⱀ (On, 150)
    b"glagolitic_pokoj",  // Ⱁ (Pokoj, 151)
    b"glagolitic_rtsy",   // Ⱂ (Rtsy, 152)
    b"glagolitic_slovo",  // Ⱃ (Slovo, 153)
    b"glagolitic_tvrido", // Ⱄ (Tvrido, 154)
    b"glagolitic_uku",    // Ⱅ (Uku, 155)
    b"glagolitic_fert",   // Ⱆ (Fert, 156)
    b"glagolitic_xrivi",  // Ⱇ (Xrivi, 157)
    b"glagolitic_ot",     // Ⱈ (Ot, 158)
    b"glagolitic_cy",     // Ⱉ (Cy, 159)
    b"glagolitic_shcha",  // Ⱊ (Shcha, 160)
    b"glagolitic_er",     // Ⱋ (Er, 161)
    b"glagolitic_yeru",   // Ⱌ (Yeru, 162)
    b"glagolitic_small_yer",// Ⱍ (Small Yer, 163)
    b"glagolitic_yo",     // Ⱎ (Yo, 164)
    b"glagolitic_yu",     // Ⱏ (Yu, 165)
    b"glagolitic_ja",     // Ⱐ (Ja, 166)
    b"thai_ko_kai",       // ก (Ko Kai, 167)
    b"thai_kho_khai",     // ข (Kho Khai, 168)
    b"thai_kho_khuat",    // ฃ (Kho Khuat, 169)
    b"thai_kho_khon",     // ค (Kho Khon, 170)
    b"thai_kho_rakhang",  // ฅ (Kho Rakhang, 171)
    b"thai_kho_khwai",    // ฆ (Kho Khwai, 172)
    b"thai_ngo_ngu",      // ง (Ngo Ngu, 173)
    b"thai_cho_chan",     // จ (Cho Chan, 174)
    b"thai_cho_ching",    // ฉ (Cho Ching, 175)
    b"thai_cho_chang",    // ช (Cho Chang, 176)
    b"thai_so_so",        // ซ (So So, 177)
    b"thai_cho_choe",     // ฌ (Cho Choe, 178)
    b"thai_yo_ying",      // ญ (Yo Ying, 179)
    b"thai_do_chada",     // ฎ (Do Chada, 180)
    b"thai_to_patak",     // ฏ (To Patak, 181)
    b"thai_tho_than",     // ฐ (Tho Than, 182)
    b"thai_tho_nangmontho",// ฑ (Tho Nangmontho, 183)
    b"thai_tho_phuthao",  // ฒ (Tho Phuthao, 184)
    b"thai_no_nen",       // ณ (No Nen, 185)
    b"thai_do_dek",       // ด (Do Dek, 186)
    b"thai_to_tao",       // ต (To Tao, 187)
    b"thai_tho_thung",    // ถ (Tho Thung, 188)
    b"thai_tho_thahan",   // ท (Tho Thahan, 189)
    b"thai_tho_thong",    // ธ (Tho Thong, 190)
    b"thai_no_nu",        // น (No Nu, 191)
    b"thai_bo_baimai",    // บ (Bo Baimai, 192)
    b"thai_po_pla",       // ป (Po Pla, 193)
    b"thai_pho_phung",    // ผ (Pho Phung, 194)
    b"thai_fo_fa",        // ฝ (Fo Fa, 195)
    b"thai_pho_phan",     // พ (Pho Phan, 196)
    b"thai_fo_fan",       // ฟ (Fo Fan, 197)
    b"thai_pho_samphao",  // ภ (Pho Samphao, 198)
    b"thai_mo_ma",        // ม (Mo Ma, 199)
    b"thai_yo_yak",       // ย (Yo Yak, 200)
    b"thai_ro_rua",       // ร (Ro Rua, 201)
    b"thai_lo_ling",      // ล (Lo Ling, 202)
    b"thai_wo_waen",      // ว (Wo Waen, 203)
    b"thai_so_sala",      // ศ (So Sala, 204)
    b"thai_so_rusi",      // ษ (So Rusi, 205)
    b"thai_so_sua",       // ส (So Sua, 206)
    b"thai_ho_hip",       // ห (Ho Hip, 207)
    b"thai_lo_chula",     // ฬ (Lo Chula, 208)
    b"thai_o_ang",        // อ (O Ang, 209)
    b"thai_ho_nokhuk",    // ฮ (Ho Nokhuk, 210)
    b"hangul_giyeok",     // ㄱ (Giyeok, 211)
    b"hangul_nieun",      // ㄴ (Nieun, 212)
    b"hangul_digeut",     // ㄷ (Digeut, 213)
    b"hangul_rieul",      // ㄹ (Rieul, 214)
    b"hangul_mieum",      // ㅁ (Mieum, 215)
    b"hangul_bieup",      // ㅂ (Bieup, 216)
    b"hangul_siot",       // ㅅ (Siot, 217)
    b"hangul_ieung",      // ㅇ (Ieung, 218)
    b"hangul_jieut",      // ㅈ (Jieut, 219)
    b"hangul_chieut",     // ㅊ (Chieut, 220)
    b"hangul_kieuk",      // ㅋ (Kieuk, 221)
    b"hangul_tieut",      // ㅌ (Tieut, 222)
    b"hangul_pieup",      // ㅍ (Pieup, 223)
    b"hangul_hieut",      // ㅎ (Hieut, 224)
    b"hangul_a",          // ㅏ (A, 225)
    b"hangul_ae",         // ㅐ (Ae, 226)
    b"hangul_ya",         // ㅑ (Ya, 227)
    b"hangul_yae",        // ㅒ (Yae, 228)
    b"hangul_eo",         // ㅓ (Eo, 229)
    b"hangul_e",          // ㅔ (E, 230)
    b"hangul_yeo",        // ㅕ (Yeo, 231)
    b"hangul_ye",         // ㅖ (Ye, 232)
    b"hangul_o",          // ㅗ (O, 233)
    b"hangul_wa",         // ㅘ (Wa, 234)
    b"hangul_wae",        // ㅙ (Wae, 235)
    b"hangul_oe",         // ㅚ (Oe, 236)
    b"hangul_yo",         // ㅛ (Yo, 237)
    b"hangul_u",          // ㅜ (U, 238)
    b"hangul_weo",        // ㅝ (Weo, 239)
    b"hangul_we",         // ㅞ (We, 240)
    b"hangul_wi",         // ㅟ (Wi, 241)
    b"hangul_yu",         // ㅠ (Yu, 242)
    b"hangul_eu",         // ㅡ (Eu, 243)
    b"hangul_ui",         // ㅢ (Ui, 244)
    b"hangul_i",          // ㅣ (I, 245)
    b"ethiopic_glottal_a",// አ (Glottal A, 246)
    b"ethiopic_glottal_u",// ኡ (Glottal U, 247)
    b"ethiopic_glottal_i",// ኢ (Glottal I, 248)
    b"ethiopic_glottal_aa",// ኣ (Glottal Aa, 249)
    b"ethiopic_glottal_e",// ኤ (Glottal E, 250)
    b"ethiopic_glottal_ie",// እ (Glottal Ie, 251)
    b"ethiopic_glottal_o",// ኦ (Glottal O, 252)
    b"ethiopic_glottal_wa",// ኧ (Glottal Wa, 253)
    b"ethiopic_wa",       // ወ (Wa, 254)
    b"ethiopic_wu",       // ዉ (Wu, 255)
    b"ethiopic_wi",       // ዊ (Wi, 256)
    b"ethiopic_waa",      // ዋ (Waa, 257)
    b"ethiopic_we",       // ዌ (We, 258)
    b"ethiopic_wye",      // ው (Wye, 259)
    b"ethiopic_wo",       // ዎ (Wo, 260)
    b"ethiopic_ko",       // ኰ (Ko, 261)
    b"ethiopic_ku",       // ኱ (Ku, 262)
    b"ethiopic_ki",       // ኲ (Ki, 263)
    b"ethiopic_kua",      // ኳ (Kua, 264)
    b"ethiopic_ke",       // ኴ (Ke, 265)
    b"ethiopic_kwe",      // ኵ (Kwe, 266)
    b"ethiopic_ko_alt",   // ኶ (Ko, 267)
    b"ethiopic_go",       // ጐ (Go, 268)
    b"ethiopic_gu",       // ጑ (Gu, 269)
    b"ethiopic_gi",       // ጒ (Gi, 270)
    b"ethiopic_gua",      // መ (Gua, 271)
    b"ethiopic_ge",       // ጔ (Ge, 272)
    b"ethiopic_gwe",      // ጕ (Gwe, 273)
    b"ethiopic_go_alt",   // ጖ (Go, 274)
    b"devanagari_a",      // अ (A, 275)
    b"devanagari_aa",     // आ (Aa, 276)
    b"devanagari_i",      // इ (I, 277)
    b"devanagari_ii",     // ई (Ii, 278)
    b"devanagari_u",      // उ (U, 279)
    b"devanagari_uu",     // ऊ (Uu, 280)
    b"devanagari_r",      // ऋ (R, 281)
    b"devanagari_e",      // ए (E, 282)
    b"devanagari_ai",     // ऐ (Ai, 283)
    b"devanagari_o",      // ओ (O, 284)
    b"devanagari_au",     // औ (Au, 285)
    b"devanagari_ka",     // क (Ka, 286)
    b"devanagari_kha",    // ख (Kha, 287)
    b"devanagari_ga",     // ग (Ga, 288)
    b"devanagari_gha",    // घ (Gha, 289)
    b"devanagari_nga",    // ङ (Nga, 290)
    b"devanagari_cha",    // च (Cha, 291)
    b"devanagari_chha",   // छ (Chha, 292)
    b"devanagari_ja",     // ज (Ja, 293)
    b"devanagari_jha",    // झ (Jha, 294)
    b"devanagari_nya",    // ञ (Nya, 295)
    b"devanagari_ta",     // ट (Ta, 296)
    b"devanagari_tha",    // ठ (Tha, 297)
    b"devanagari_da",     // ड (Da, 298)
    b"devanagari_dha",    // ढ (Dha, 299)
    b"devanagari_na",     // ण (Na, 300)
    b"devanagari_ta_alt", // त (Ta, 301)
    b"devanagari_tha_alt",// थ (Tha, 302)
    b"devanagari_da_alt", // द (Da, 303)
    b"devanagari_dha_alt",// ध (Dha, 304)
    b"devanagari_na_alt", // न (Na, 305)
    b"devanagari_pa",     // प (Pa, 306)
    b"devanagari_pha",    // फ (Pha, 307)
    b"devanagari_ba",     // ब (Ba, 308)
    b"devanagari_bha",    // भ (Bha, 309)
    b"devanagari_ma",     // म (Ma, 310)
    b"devanagari_ya",     // य (Ya, 311)
    b"devanagari_ra",     // र (Ra, 312)
    b"devanagari_la",     // ल (La, 313)
    b"devanagari_va",     // व (Va, 314)
    b"devanagari_sha",    // श (Sha, 315)
    b"devanagari_ssa",    // ष (Ssa, 316)
    b"devanagari_sa",     // स (Sa, 317)
    b"devanagari_ha",     // ह (Ha, 318)
    b"katakana_a",        // ア (A, 319)
    b"kana_i",
    b"kana_u",
    b"kana_e",
    b"kana_o",
    b"kana_a",
    b"kana_ki",
    b"kana_ku",
    b"kana_ke",
    b"kana_ko",
    b"kana_sa",
    b"kana_shi",
    b"kana_su",
    b"kana_se",
    b"kana_so",
    b"kana_ta",
    b"kana_chi",
    b"kana_tsu",
    b"kana_te",
    b"kana_to",
    b"kana_na",
    b"kana_ni",
    b"kana_nu",
    b"kana_ne",
    b"kana_no",
    b"kana_ha",
    b"kana_hi",
    b"kana_fu",
    b"kana_he",
    b"kana_ho",
    b"kana_ma",
    b"kana_mi",
    b"kana_mu",
    b"kana_me",
    b"kana_mo",
    b"kana_ya",
    b"kana_yu",
    b"kana_yo",
    b"kana_ra",
    b"kana_ri",
    b"kana_ru",
    b"kana_re",
    b"kana_ro",
    b"kana_wa",
    b"kana_wo",
    b"kana_n",
    b"ya",
    b"yab",
    b"yabh",
    b"yag",
    b"yagh",
    b"yaj",
    b"yach",
    b"yad",
    b"yadh",
    b"yadhe",
    b"yaz",
    b"yazh",
    b"yaf",
    b"yak",
    b"yakv",
    b"yaq",
    b"yah",
    b"yahh",
    b"yahl",
    b"yahm",
    b"yayn",
    b"yakh",
    b"yakl",
    b"yahq",
    b"yash",
    b"yi",
    b"yij",
    b"yizh",
    b"yink",
    b"yal",
    b"yam",
    b"yan",
    b"yang",
    b"yany",
    b"yap",
    b"yu",
    b"a",
    b"aa",
    b"i",
    b"ii",
    b"u",
    b"uu",
    b"r",
    b"rr",
    b"l",
    b"ll",
    b"e",
    b"ee",
    b"ai",
    b"o",
    b"oo",
    b"au",
    b"ka",
    b"kha",
    b"ga",
    b"gha",
    b"nga",
    b"cha",
    b"chha",
    b"ja",
    b"jha",
    b"nya",
    b"ta",
    b"tha",
    b"da",
    b"dha",
    b"na",
    b"pa",
    b"pha",
    b"ba",
    b"bha",
    b"ma",
    b"ya",
    b"ra",
];

impl<T: Config> Pallet<T> {
    /// Returns the human-readable name for a subnet as a `Vec<u8>`.
    ///
    /// Priority:
    /// 1. If the subnet has a non-empty `SubnetIdentitiesV3::subnet_name` set by
    ///    its owner, return that.
    /// 2. Otherwise fall back to the built-in `NAMES` table, which mirrors
    ///    `SYMBOLS` by `netuid`.
    /// 3. Return `b"unknown"` if the netuid is out of range.
    pub fn get_name_for_subnet(netuid: NetUid) -> Vec<u8> {
        SubnetIdentitiesV3::<T>::try_get(netuid)
            .and_then(|identity| {
                if !identity.subnet_name.is_empty() {
                    Ok(identity.subnet_name)
                } else {
                    Err(())
                }
            })
            .unwrap_or_else(|_| {
                NAMES
                    .get(u16::from(netuid) as usize)
                    .copied()
                    .unwrap_or(b"unknown")
                    .to_vec()
            })
    }

    pub fn get_symbol_for_subnet(netuid: NetUid) -> Vec<u8> {
        SYMBOLS
            .get(u16::from(netuid) as usize)
            .unwrap_or(&DEFAULT_SYMBOL)
            .to_vec()
    }

    pub fn get_next_available_symbol(netuid: NetUid) -> Vec<u8> {
        let used_symbols: BTreeSet<Vec<u8>> = TokenSymbol::<T>::iter_values().collect();

        // We first try the default strategy of using the subnet id to get the symbol.
        let symbol = Self::get_symbol_for_subnet(netuid);
        if !used_symbols.contains(&symbol) {
            return symbol;
        }

        // If it is already taken, we try to get the next available symbol.
        let available_symbol = SYMBOLS
            .iter()
            .skip(1) // Skip the root symbol
            .find(|s| !used_symbols.contains(&s[..]))
            .map(|s| s.to_vec());

        if available_symbol.is_none() {
            log::warn!(
                "All available symbols have been exhausted for netuid: {netuid:?}. Using default symbol."
            );
        }

        // If we have exhausted all symbols, we use the default symbol.
        available_symbol.unwrap_or(DEFAULT_SYMBOL.to_vec())
    }

    pub fn ensure_symbol_exists(symbol: &[u8]) -> DispatchResult {
        if !SYMBOLS.iter().skip(1).any(|s| s == &symbol) {
            return Err(Error::<T>::SymbolDoesNotExist.into());
        }

        Ok(())
    }

    pub fn ensure_symbol_available(symbol: &[u8]) -> DispatchResult {
        if TokenSymbol::<T>::iter_values().any(|s| s == symbol) {
            return Err(Error::<T>::SymbolAlreadyInUse.into());
        }

        Ok(())
    }
}
