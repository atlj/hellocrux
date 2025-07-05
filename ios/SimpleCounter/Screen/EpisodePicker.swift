//
//  EpisodePicker.swift
//  SimpleCounter
//
//  Created by Burak Güner on 5.07.2025.
//

import SwiftUI

struct EpisodePicker: View {
    let id: String
    let series: [UInt32: [UInt32: String]]
    @State private var season = 1
    @EnvironmentObject var core: Core

    var data: [Season] {
        var seasons = [Season]()
        
        for (seasonNumber, episodes) in series {
            var children = [Season]()
            for (episodeNumber, source) in episodes {
                children.append(Season(data: .episode(Int(seasonNumber), Int(episodeNumber), source)))
            }
            children.sort { $0.number < $1.number }
            seasons.append(Season(data: .season((Int(seasonNumber))), children: children))
        }
        
        seasons.sort { $0.number < $1.number }
        return seasons
    }
    
    var body: some View {
        List {
            OutlineGroup(data, children: \.children) { item in
                switch item.data {
                case let .season(seasonId):
                    Text("Season \(seasonId)")
                case let .episode(seasonId, episodeId, source):
                    Button {
                        core.update(.play(.fromCertainEpisode(id: id, episode: .init(season: UInt32(seasonId), episode: UInt32(episodeId)))))
                    } label: {
                        Text("Episode \(episodeId)")
                    }
                }
            }
            .listRowBackground(Color.clear)
        }.background(.clear)
    }
}

#Preview {
    EpisodePicker(
        id: "test", series: [
            1: [
                8: "",
                1: "",
                2: ""
            ],
            2: [
                :
            ]
        ]
    )
    .environmentObject(Core())
}

struct Season: Hashable, Identifiable {
    var id: Self { self }
    var data: SeasonData
    var children: [Season]? = nil
    
    var number: Int {
        switch self.data {
        case let .season(seasonNumber):
            return seasonNumber
        case let .episode(_, episodeNumber, _):
            return episodeNumber
        }
    }
}

enum SeasonData : Hashable {
    case season(Int)
    case episode(Int, Int, String)
}
