import SharedTypes
import SwiftUI

struct LanguageSelectorSheet: View {
    @Environment(\.dismiss) private var dismiss
    @Binding var selectedLanguage: LanguageCode
    @State var search = ""
    var items: [LanguageCode] {
        if search.isEmpty {
            return LanguageCode.allCases
        }

        return LanguageCode.allCases.filter { code in
            Locale.current.localizedString(forLanguageCode: code.iso639_2t())!.lowercased().contains(search.lowercased())
        }
    }

    var body: some View {
        NavigationStack {
            Form {
                ForEach(items, id: \.hashValue) { language in
                    Button(Locale.current.localizedString(forLanguageCode: language.iso639_2t())!) {
                        selectedLanguage = language
                        dismiss.callAsFunction()
                    }
                }
            }
            .searchable(text: $search, prompt: "Search")
            .navigationTitle("Select a Language")
            .navigationBarTitleDisplayMode(.inline)
        }
    }
}

@available(iOS 17.0, *)
#Preview {
    @Previewable @State var language = LanguageCode.turkish
    LanguageSelectorSheet(selectedLanguage: $language)
}

extension LanguageCode: @retroactive CaseIterable {
    public static var allCases: [LanguageCode] = [
        .abkhazian,
        .afar,
        .afrikaans,
        .akan,
        .albanian,
        .amharic,
        .arabic,
        .aragonese,
        .armenian,
        .assamese,
        .avaric,
        .avestan,
        .aymara,
        .azerbaijani,
        .bambara,
        .bashkir,
        .basque,
        .belarusian,
        .bengali,
        .bislama,
        .bosnian,
        .breton,
        .bulgarian,
        .burmese,
        .catalan,
        .chamorro,
        .chechen,
        .chichewa,
        .chinese,
        .churchSlavonic,
        .chuvash,
        .cornish,
        .corsican,
        .cree,
        .croatian,
        .czech,
        .danish,
        .divehi,
        .dutch,
        .dzongkha,
        .english,
        .esperanto,
        .estonian,
        .ewe,
        .faroese,
        .fijian,
        .finnish,
        .french,
        .westernFrisian,
        .fulah,
        .gaelic,
        .galician,
        .ganda,
        .georgian,
        .german,
        .greek,
        .kalaallisut,
        .guarani,
        .gujarati,
        .haitian,
        .hausa,
        .hebrew,
        .herero,
        .hindi,
        .hiriMotu,
        .hungarian,
        .icelandic,
        .ido,
        .igbo,
        .indonesian,
        .interlingua,
        .interlingue,
        .inuktitut,
        .inupiaq,
        .irish,
        .italian,
        .japanese,
        .javanese,
        .kannada,
        .kanuri,
        .kashmiri,
        .kazakh,
        .centralKhmer,
        .kikuyu,
        .kinyarwanda,
        .kyrgyz,
        .komi,
        .kongo,
        .korean,
        .kuanyama,
        .kurdish,
        .lao,
        .latin,
        .latvian,
        .limburgan,
        .lingala,
        .lithuanian,
        .lubaKatanga,
        .luxembourgish,
        .macedonian,
        .malagasy,
        .malay,
        .malayalam,
        .maltese,
        .manx,
        .maori,
        .marathi,
        .marshallese,
        .mongolian,
        .nauru,
        .navajo,
        .northNdebele,
        .southNdebele,
        .ndonga,
        .nepali,
        .norwegian,
        .norwegianBokmål,
        .norwegianNynorsk,
        .occitan,
        .ojibwa,
        .oriya,
        .oromo,
        .ossetian,
        .pali,
        .pashto,
        .persian,
        .polish,
        .portuguese,
        .punjabi,
        .quechua,
        .romanian,
        .romansh,
        .rundi,
        .russian,
        .northernSami,
        .samoan,
        .sango,
        .sanskrit,
        .sardinian,
        .serbian,
        .shona,
        .sindhi,
        .sinhala,
        .slovak,
        .slovenian,
        .somali,
        .southernSotho,
        .spanish,
        .sundanese,
        .swahili,
        .swati,
        .swedish,
        .tagalog,
        .tahitian,
        .tajik,
        .tamil,
        .tatar,
        .telugu,
        .thai,
        .tibetan,
        .tigrinya,
        .tonga,
        .tsonga,
        .tswana,
        .turkish,
        .turkmen,
        .twi,
        .uighur,
        .ukrainian,
        .urdu,
        .uzbek,
        .venda,
        .vietnamese,
        .volapük,
        .walloon,
        .welsh,
        .wolof,
        .xhosa,
        .sichuanYi,
        .yiddish,
        .yoruba,
        .zhuang,
        .zulu,
    ]
}

extension LanguageCode {
    func iso639_2t() -> String {
        switch self {
        case .abkhazian:
            "abk"
        case .afar:
            "aar"
        case .afrikaans:
            "afr"
        case .akan:
            "aka"
        case .albanian:
            "sqi"
        case .amharic:
            "amh"
        case .arabic:
            "ara"
        case .aragonese:
            "arg"
        case .armenian:
            "hye"
        case .assamese:
            "asm"
        case .avaric:
            "ava"
        case .avestan:
            "ave"
        case .aymara:
            "aym"
        case .azerbaijani:
            "aze"
        case .bambara:
            "bam"
        case .bashkir:
            "bak"
        case .basque:
            "eus"
        case .belarusian:
            "bel"
        case .bengali:
            "ben"
        case .bislama:
            "bis"
        case .bosnian:
            "bos"
        case .breton:
            "bre"
        case .bulgarian:
            "bul"
        case .burmese:
            "mya"
        case .catalan:
            "cat"
        case .chamorro:
            "cha"
        case .chechen:
            "che"
        case .chichewa:
            "nya"
        case .chinese:
            "zho"
        case .churchSlavonic:
            "chu"
        case .chuvash:
            "chv"
        case .cornish:
            "cor"
        case .corsican:
            "cos"
        case .cree:
            "cre"
        case .croatian:
            "hrv"
        case .czech:
            "ces"
        case .danish:
            "dan"
        case .divehi:
            "div"
        case .dutch:
            "nld"
        case .dzongkha:
            "dzo"
        case .english:
            "eng"
        case .esperanto:
            "epo"
        case .estonian:
            "est"
        case .ewe:
            "ewe"
        case .faroese:
            "fao"
        case .fijian:
            "fij"
        case .finnish:
            "fin"
        case .french:
            "fra"
        case .westernFrisian:
            "fry"
        case .fulah:
            "ful"
        case .gaelic:
            "gla"
        case .galician:
            "glg"
        case .ganda:
            "lug"
        case .georgian:
            "kat"
        case .german:
            "deu"
        case .greek:
            "ell"
        case .kalaallisut:
            "kal"
        case .guarani:
            "grn"
        case .gujarati:
            "guj"
        case .haitian:
            "hat"
        case .hausa:
            "hau"
        case .hebrew:
            "heb"
        case .herero:
            "her"
        case .hindi:
            "hin"
        case .hiriMotu:
            "hmo"
        case .hungarian:
            "hun"
        case .icelandic:
            "isl"
        case .ido:
            "ido"
        case .igbo:
            "ibo"
        case .indonesian:
            "ind"
        case .interlingua:
            "ina"
        case .interlingue:
            "ile"
        case .inuktitut:
            "iku"
        case .inupiaq:
            "ipk"
        case .irish:
            "gle"
        case .italian:
            "ita"
        case .japanese:
            "jpn"
        case .javanese:
            "jav"
        case .kannada:
            "kan"
        case .kanuri:
            "kau"
        case .kashmiri:
            "kas"
        case .kazakh:
            "kaz"
        case .centralKhmer:
            "khm"
        case .kikuyu:
            "kik"
        case .kinyarwanda:
            "kin"
        case .kyrgyz:
            "kir"
        case .komi:
            "kom"
        case .kongo:
            "kon"
        case .korean:
            "kor"
        case .kuanyama:
            "kua"
        case .kurdish:
            "kur"
        case .lao:
            "lao"
        case .latin:
            "lat"
        case .latvian:
            "lav"
        case .limburgan:
            "lim"
        case .lingala:
            "lin"
        case .lithuanian:
            "lit"
        case .lubaKatanga:
            "lub"
        case .luxembourgish:
            "ltz"
        case .macedonian:
            "mkd"
        case .malagasy:
            "mlg"
        case .malay:
            "msa"
        case .malayalam:
            "mal"
        case .maltese:
            "mlt"
        case .manx:
            "glv"
        case .maori:
            "mri"
        case .marathi:
            "mar"
        case .marshallese:
            "mah"
        case .mongolian:
            "mon"
        case .nauru:
            "nau"
        case .navajo:
            "nav"
        case .northNdebele:
            "nde"
        case .southNdebele:
            "nbl"
        case .ndonga:
            "ndo"
        case .nepali:
            "nep"
        case .norwegian:
            "nor"
        case .norwegianBokmål:
            "nob"
        case .norwegianNynorsk:
            "nno"
        case .occitan:
            "oci"
        case .ojibwa:
            "oji"
        case .oriya:
            "ori"
        case .oromo:
            "orm"
        case .ossetian:
            "oss"
        case .pali:
            "pli"
        case .pashto:
            "pus"
        case .persian:
            "fas"
        case .polish:
            "pol"
        case .portuguese:
            "por"
        case .punjabi:
            "pan"
        case .quechua:
            "que"
        case .romanian:
            "ron"
        case .romansh:
            "roh"
        case .rundi:
            "run"
        case .russian:
            "rus"
        case .northernSami:
            "sme"
        case .samoan:
            "smo"
        case .sango:
            "sag"
        case .sanskrit:
            "san"
        case .sardinian:
            "srd"
        case .serbian:
            "srp"
        case .shona:
            "sna"
        case .sindhi:
            "snd"
        case .sinhala:
            "sin"
        case .slovak:
            "slk"
        case .slovenian:
            "slv"
        case .somali:
            "som"
        case .southernSotho:
            "sot"
        case .spanish:
            "spa"
        case .sundanese:
            "sun"
        case .swahili:
            "swa"
        case .swati:
            "ssw"
        case .swedish:
            "swe"
        case .tagalog:
            "tgl"
        case .tahitian:
            "tah"
        case .tajik:
            "tgk"
        case .tamil:
            "tam"
        case .tatar:
            "tat"
        case .telugu:
            "tel"
        case .thai:
            "tha"
        case .tibetan:
            "bod"
        case .tigrinya:
            "tir"
        case .tonga:
            "ton"
        case .tsonga:
            "tso"
        case .tswana:
            "tsn"
        case .turkish:
            "tur"
        case .turkmen:
            "tuk"
        case .twi:
            "twi"
        case .uighur:
            "uig"
        case .ukrainian:
            "ukr"
        case .urdu:
            "urd"
        case .uzbek:
            "uzb"
        case .venda:
            "ven"
        case .vietnamese:
            "vie"
        case .volapük:
            "vol"
        case .walloon:
            "wln"
        case .welsh:
            "cym"
        case .wolof:
            "wol"
        case .xhosa:
            "xho"
        case .sichuanYi:
            "iii"
        case .yiddish:
            "yid"
        case .yoruba:
            "yor"
        case .zhuang:
            "zha"
        case .zulu:
            "zul"
        }
    }
}
