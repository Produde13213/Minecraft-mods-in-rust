//! Iris-style shader program inventory and fallback resolution.
//! Clean-room compatibility layer for public shader-pack naming conventions.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Program {
    Shadow, ShadowSolid, ShadowCutout, ShadowWater, ShadowEntities, ShadowLightning, ShadowBlock,
    Basic, Line, Textured, TexturedLit, SkyBasic, SkyTextured, Clouds, Terrain, TerrainSolid,
    TerrainCutout, DamagedBlock, Block, BlockTranslucent, BeaconBeam, Item, Entities,
    EntitiesTranslucent, Lightning, Particles, ParticlesTranslucent, EntitiesGlowing,
    ArmorGlint, SpiderEyes, Hand, Weather, Water, HandWater, DhTerrain, DhWater, DhGeneric,
    DhShadow, Final,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramGroup { Shadow, Gbuffers, DistantHorizons, Final }

impl Program {
    pub const ALL: &'static [Program] = &[
        Self::Shadow, Self::ShadowSolid, Self::ShadowCutout, Self::ShadowWater,
        Self::ShadowEntities, Self::ShadowLightning, Self::ShadowBlock,
        Self::Basic, Self::Line, Self::Textured, Self::TexturedLit, Self::SkyBasic,
        Self::SkyTextured, Self::Clouds, Self::Terrain, Self::TerrainSolid,
        Self::TerrainCutout, Self::DamagedBlock, Self::Block, Self::BlockTranslucent,
        Self::BeaconBeam, Self::Item, Self::Entities, Self::EntitiesTranslucent,
        Self::Lightning, Self::Particles, Self::ParticlesTranslucent, Self::EntitiesGlowing,
        Self::ArmorGlint, Self::SpiderEyes, Self::Hand, Self::Weather, Self::Water,
        Self::HandWater, Self::DhTerrain, Self::DhWater, Self::DhGeneric, Self::DhShadow,
        Self::Final,
    ];

    pub const fn group(self) -> ProgramGroup {
        match self {
            Self::Shadow | Self::ShadowSolid | Self::ShadowCutout | Self::ShadowWater
            | Self::ShadowEntities | Self::ShadowLightning | Self::ShadowBlock => ProgramGroup::Shadow,
            Self::DhTerrain | Self::DhWater | Self::DhGeneric | Self::DhShadow => ProgramGroup::DistantHorizons,
            Self::Final => ProgramGroup::Final,
            _ => ProgramGroup::Gbuffers,
        }
    }

    pub const fn source_name(self) -> &'static str {
        match self {
            Self::Shadow => "shadow", Self::ShadowSolid => "shadow_solid",
            Self::ShadowCutout => "shadow_cutout", Self::ShadowWater => "shadow_water",
            Self::ShadowEntities => "shadow_entities", Self::ShadowLightning => "shadow_lightning",
            Self::ShadowBlock => "shadow_block", Self::Basic => "gbuffers_basic",
            Self::Line => "gbuffers_line", Self::Textured => "gbuffers_textured",
            Self::TexturedLit => "gbuffers_textured_lit", Self::SkyBasic => "gbuffers_skybasic",
            Self::SkyTextured => "gbuffers_skytextured", Self::Clouds => "gbuffers_clouds",
            Self::Terrain => "gbuffers_terrain", Self::TerrainSolid => "gbuffers_terrain_solid",
            Self::TerrainCutout => "gbuffers_terrain_cutout", Self::DamagedBlock => "gbuffers_damagedblock",
            Self::Block => "gbuffers_block", Self::BlockTranslucent => "gbuffers_block_translucent",
            Self::BeaconBeam => "gbuffers_beaconbeam", Self::Item => "gbuffers_item",
            Self::Entities => "gbuffers_entities", Self::EntitiesTranslucent => "gbuffers_entities_translucent",
            Self::Lightning => "gbuffers_lightning", Self::Particles => "gbuffers_particles",
            Self::ParticlesTranslucent => "gbuffers_particles_translucent",
            Self::EntitiesGlowing => "gbuffers_entities_glowing", Self::ArmorGlint => "gbuffers_armor_glint",
            Self::SpiderEyes => "gbuffers_spidereyes", Self::Hand => "gbuffers_hand",
            Self::Weather => "gbuffers_weather", Self::Water => "gbuffers_water",
            Self::HandWater => "gbuffers_hand_water", Self::DhTerrain => "dh_terrain",
            Self::DhWater => "dh_water", Self::DhGeneric => "dh_generic",
            Self::DhShadow => "dh_shadow", Self::Final => "final",
        }
    }

    pub const fn fallback(self) -> Option<Program> {
        match self {
            Self::ShadowSolid | Self::ShadowCutout | Self::ShadowWater | Self::ShadowEntities
            | Self::ShadowBlock => Some(Self::Shadow),
            Self::ShadowLightning => Some(Self::ShadowEntities),
            Self::Line | Self::Textured | Self::SkyBasic => Some(Self::Basic),
            Self::TexturedLit => Some(Self::Textured),
            Self::SkyTextured | Self::Clouds | Self::BeaconBeam | Self::ArmorGlint
            | Self::SpiderEyes => Some(Self::Textured),
            Self::Terrain => Some(Self::TexturedLit),
            Self::TerrainSolid | Self::TerrainCutout | Self::DamagedBlock | Self::Block => Some(Self::Terrain),
            Self::BlockTranslucent => Some(Self::Block),
            Self::Item | Self::Entities | Self::Particles | Self::Hand | Self::Weather => Some(Self::TexturedLit),
            Self::EntitiesTranslucent | Self::Lightning | Self::EntitiesGlowing => Some(Self::Entities),
            Self::ParticlesTranslucent => Some(Self::Particles),
            Self::Water => Some(Self::Terrain),
            Self::HandWater => Some(Self::Hand),
            Self::DhWater | Self::DhGeneric => Some(Self::DhTerrain),
            _ => None,
        }
    }

    pub fn stage_paths(self) -> [String; 2] {
        let base = self.source_name();
        [format!("shaders/{base}.vsh"), format!("shaders/{base}.fsh")]
    }

    pub fn fallback_chain(self) -> Vec<Program> {
        let mut chain = Vec::new();
        let mut current = Some(self);
        while let Some(program) = current {
            if chain.contains(&program) { break; }
            chain.push(program);
            current = program.fallback();
        }
        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_program_names_to_shader_pack_files() {
        assert_eq!(Program::Terrain.source_name(), "gbuffers_terrain");
        assert_eq!(Program::Final.stage_paths(), [
            "shaders/final.vsh".to_owned(), "shaders/final.fsh".to_owned()
        ]);
        assert_eq!(Program::DhShadow.group(), ProgramGroup::DistantHorizons);
    }

    #[test]
    fn specialized_programs_have_ordered_fallbacks() {
        assert_eq!(Program::TerrainCutout.fallback_chain(), [
            Program::TerrainCutout, Program::Terrain, Program::TexturedLit,
            Program::Textured, Program::Basic
        ]);
        assert_eq!(Program::ShadowLightning.fallback_chain(), [
            Program::ShadowLightning, Program::ShadowEntities, Program::Shadow
        ]);
    }

    #[test]
    fn every_program_has_a_unique_source_name() {
        for (index, program) in Program::ALL.iter().enumerate() {
            assert!(!Program::ALL[..index].iter().any(|p| p.source_name() == program.source_name()));
        }
    }
}
