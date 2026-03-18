use super::ast::*;
use super::lexer::{Token, TokenKind};
use super::types::*;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug, thiserror::Error)]
#[error("parse error at line {line}: {message}")]
pub struct ParseError {
    pub message: String,
    pub line: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        match self.peek_kind() {
            TokenKind::House => Ok(Program::House(self.parse_house()?)),
            TokenKind::Furniture => Ok(Program::Furniture(self.parse_furniture()?)),
            other => Err(self.error(&format!("expected 'house' or 'furniture', got {other}"))),
        }
    }

    // --- House ---

    fn parse_house(&mut self) -> Result<HouseBlock, ParseError> {
        self.expect(TokenKind::House)?;
        let name = self.try_string_or_ident();
        self.expect(TokenKind::LBrace)?;

        let mut house = HouseBlock {
            name,
            site: None,
            style: None,
            floors: Vec::new(),
            roof: None,
            facades: Vec::new(),
            landscape: None,
        };

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            match self.peek_kind() {
                TokenKind::Site => house.site = Some(self.parse_site()?),
                TokenKind::Style => house.style = Some(self.parse_style()?),
                TokenKind::Floor => house.floors.push(self.parse_floor()?),
                TokenKind::Roof => house.roof = Some(self.parse_roof()?),
                TokenKind::Facade => house.facades.push(self.parse_facade()?),
                TokenKind::Landscape => house.landscape = Some(self.parse_landscape()?),
                other => {
                    return Err(self.error(&format!(
                        "unexpected {other} in house block"
                    )))
                }
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(house)
    }

    // --- Site ---

    fn parse_site(&mut self) -> Result<SiteBlock, ParseError> {
        self.expect(TokenKind::Site)?;
        self.expect(TokenKind::LBrace)?;

        let mut site = SiteBlock {
            footprint: None,
            orientation: None,
            setbacks: Vec::new(),
            slope: None,
            garage_access: None,
        };

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let key = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            match key.as_str() {
                "footprint" => {
                    let w = self.parse_dimension()?;
                    self.expect(TokenKind::X)?;
                    let d = self.parse_dimension()?;
                    site.footprint = Some((w, d));
                }
                "orientation" => {
                    let val = self.expect_ident()?;
                    site.orientation = val.parse().ok();
                }
                "setback" => {
                    loop {
                        let name = self.expect_ident()?;
                        let dim = self.parse_dimension()?;
                        site.setbacks.push((name, dim));
                        if !self.try_consume(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                "slope" => site.slope = Some(self.consume_value_text()),
                "garage_access" => site.garage_access = Some(self.consume_value_text()),
                _ => {
                    self.consume_value_text();
                }
            }
            self.try_consume(TokenKind::Comma);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(site)
    }

    // --- Style ---

    fn parse_style(&mut self) -> Result<StyleBlock, ParseError> {
        self.expect(TokenKind::Style)?;
        let name = self.expect_ident_or_string()?;

        let parent = if self.try_consume(TokenKind::Colon) {
            Some(self.expect_ident()?)
        } else {
            None
        };

        let mut overrides = Vec::new();
        if self.check(TokenKind::LBrace) {
            self.advance();
            while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
                let key = self.expect_ident()?;
                self.expect(TokenKind::Colon)?;
                let value = self.consume_value_text();
                overrides.push((key, value));
                self.try_consume(TokenKind::Comma);
            }
            self.expect(TokenKind::RBrace)?;
        }

        Ok(StyleBlock {
            name,
            parent,
            overrides,
        })
    }

    // --- Floor ---

    fn parse_floor(&mut self) -> Result<FloorBlock, ParseError> {
        self.expect(TokenKind::Floor)?;
        let name = self.expect_ident_or_string()?;
        self.expect(TokenKind::LBrace)?;

        let mut floor = FloorBlock {
            name,
            ceiling_height: None,
            rooms: Vec::new(),
        };

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            match self.peek_kind() {
                TokenKind::Room => floor.rooms.push(self.parse_room()?),
                TokenKind::Ident(_) => {
                    let key = self.expect_ident()?;
                    self.expect(TokenKind::Colon)?;
                    if key == "ceiling_height" {
                        floor.ceiling_height = Some(self.parse_dimension()?);
                    } else {
                        self.consume_value_text();
                    }
                    self.try_consume(TokenKind::Comma);
                }
                other => {
                    return Err(self.error(&format!("unexpected {other} in floor block")))
                }
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(floor)
    }

    // --- Room ---

    fn parse_room(&mut self) -> Result<RoomBlock, ParseError> {
        self.expect(TokenKind::Room)?;
        let name = self.expect_ident_or_string()?;
        self.expect(TokenKind::LBrace)?;

        let mut room = RoomBlock {
            name,
            area: None,
            aspect: None,
            adjacent_to: Vec::new(),
            connects: Vec::new(),
            windows: Vec::new(),
            features: Vec::new(),
            side: None,
            ceiling: None,
            flooring: None,
            purpose: None,
        };

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let key = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            match key.as_str() {
                "area" => room.area = Some(self.parse_approx_value()?),
                "aspect" => room.aspect = Some(self.parse_approx_value()?),
                "adjacent_to" => room.adjacent_to = self.parse_ref_list()?,
                "connects" => room.connects = self.parse_ref_list()?,
                "windows" => room.windows = self.parse_window_specs()?,
                "has" => room.features = self.parse_feature_list()?,
                "side" => {
                    let val = self.expect_ident()?;
                    room.side = val.parse().ok();
                }
                "ceiling" => {
                    let val = self.expect_ident()?;
                    room.ceiling = val.parse().ok();
                }
                "flooring" => {
                    let val = self.consume_value_text();
                    room.flooring = Some(MaterialSpec {
                        material_type: val,
                        color: None,
                    });
                }
                "purpose" => {
                    let val = self.expect_ident()?;
                    room.purpose = Some(RoomType::infer_from_name(&val));
                }
                _ => {
                    self.consume_value_text();
                }
            }
            self.try_consume(TokenKind::Comma);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(room)
    }

    // --- Roof ---

    fn parse_roof(&mut self) -> Result<RoofBlock, ParseError> {
        self.expect(TokenKind::Roof)?;
        self.expect(TokenKind::LBrace)?;

        let mut roof = RoofBlock {
            primary: None,
            cross_gable: None,
            dormers: None,
            material: None,
            pitch: None,
            overhang: None,
        };

        // Roof properties have complex inline syntax (e.g., "dormers: 2, over [bedroom_2, bedroom_3]")
        // so we consume entire lines as value text, stopping only at newline-level boundaries.
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let key = self.expect_ident()?;
            // Some roof sub-properties (like "over") don't have a colon — they're continuations
            // of the previous property. Collect them into the previous value.
            if !self.check(TokenKind::Colon) {
                // This is a continuation like "over [bedroom_2, bedroom_3]"
                let rest = self.consume_value_text();
                let combined = format!("{key} {rest}");
                // Append to dormers/cross_gable if they exist
                if let Some(ref mut d) = roof.dormers {
                    *d = format!("{d}, {combined}");
                } else if let Some(ref mut cg) = roof.cross_gable {
                    *cg = format!("{cg}, {combined}");
                }
                self.try_consume(TokenKind::Comma);
                continue;
            }
            self.expect(TokenKind::Colon)?;
            match key.as_str() {
                "primary" => roof.primary = Some(self.consume_value_text()),
                "cross_gable" => roof.cross_gable = Some(self.consume_value_text()),
                "dormers" => roof.dormers = Some(self.consume_value_text()),
                "material" => {
                    let val = self.consume_value_text();
                    roof.material = Some(MaterialSpec {
                        material_type: val,
                        color: None,
                    });
                }
                "pitch" => roof.pitch = Some(self.consume_value_text()),
                "overhang" => roof.overhang = self.try_parse_dimension(),
                _ => {
                    self.consume_value_text();
                }
            }
            self.try_consume(TokenKind::Comma);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(roof)
    }

    // --- Facade ---

    fn parse_facade(&mut self) -> Result<FacadeBlock, ParseError> {
        self.expect(TokenKind::Facade)?;
        let side = self.expect_ident_or_string()?;
        self.expect(TokenKind::LBrace)?;

        let mut properties = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let key = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let value = self.consume_value_text();
            properties.push((key, value));
            self.try_consume(TokenKind::Comma);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(FacadeBlock { side, properties })
    }

    // --- Landscape ---

    fn parse_landscape(&mut self) -> Result<LandscapeBlock, ParseError> {
        self.expect(TokenKind::Landscape)?;
        self.expect(TokenKind::LBrace)?;

        let mut properties = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let key = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let value = self.consume_value_text();
            properties.push((key, value));
            self.try_consume(TokenKind::Comma);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(LandscapeBlock { properties })
    }

    // --- Furniture ---

    fn parse_furniture(&mut self) -> Result<FurnitureBlock, ParseError> {
        self.expect(TokenKind::Furniture)?;
        let name = self.expect_ident_or_string()?;
        self.expect(TokenKind::LBrace)?;

        let mut properties = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let key = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let value = self.consume_value_text();
            properties.push((key, value));
            self.try_consume(TokenKind::Comma);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(FurnitureBlock { name, properties })
    }

    // --- Helpers ---

    fn parse_dimension(&mut self) -> Result<Dimension, ParseError> {
        let num = self.expect_number()?;
        let unit_str = self.expect_unit()?;
        let unit = unit_str
            .parse::<Unit>()
            .map_err(|e| self.error(&e))?;
        Ok(Dimension::new(num, unit))
    }

    fn try_parse_dimension(&mut self) -> Option<Dimension> {
        if let TokenKind::Number(_) = self.peek_kind() {
            self.parse_dimension().ok()
        } else {
            None
        }
    }

    fn parse_approx_value(&mut self) -> Result<ApproxValue, ParseError> {
        // ~25sqm  |  20sqm..30sqm  |  25sqm  |  1.2..1.8  |  large
        if self.try_consume(TokenKind::Tilde) {
            let num = self.expect_number()?;
            if let Some(unit_str) = self.try_unit() {
                let unit = unit_str.parse::<Unit>().map_err(|e| self.error(&e))?;
                return Ok(ApproxValue::Approximate(num, unit));
            }
            // Approximate without unit (e.g., ~1.5 for aspect)
            return Ok(ApproxValue::Approximate(num, Unit::Mm));
        }

        // Check for qualitative
        if let TokenKind::Ident(s) = self.peek_kind() {
            if matches!(s.as_str(), "large" | "small" | "medium" | "extra_large") {
                let word = s.clone();
                self.advance();
                return Ok(ApproxValue::Qualitative(word));
            }
        }

        let num = self.expect_number()?;
        let unit_str = self.try_unit();

        // Check for range
        if self.check(TokenKind::DotDot) {
            self.advance();
            let num2 = self.expect_number()?;
            let unit_str2 = self.try_unit();
            let unit = unit_str
                .or(unit_str2)
                .and_then(|s| s.parse::<Unit>().ok())
                .unwrap_or(Unit::Mm);
            return Ok(ApproxValue::Range(num, num2, unit));
        }

        if let Some(u) = unit_str {
            let unit = u.parse::<Unit>().map_err(|e| self.error(&e))?;
            Ok(ApproxValue::Exact(num, unit))
        } else {
            // Plain number (e.g., aspect ratio)
            Ok(ApproxValue::Exact(num, Unit::Mm))
        }
    }

    fn parse_ref_list(&mut self) -> Result<Vec<String>, ParseError> {
        // Either a single ident or [ident, ident, ...]
        if self.try_consume(TokenKind::LBracket) {
            let mut refs = Vec::new();
            while !self.check(TokenKind::RBracket) && !self.check(TokenKind::Eof) {
                refs.push(self.expect_ident()?);
                self.try_consume(TokenKind::Comma);
            }
            self.expect(TokenKind::RBracket)?;
            Ok(refs)
        } else {
            // Single ref
            Ok(vec![self.expect_ident()?])
        }
    }

    fn parse_feature_list(&mut self) -> Result<Vec<Feature>, ParseError> {
        let refs = self.parse_ref_list()?;
        let mut features = Vec::new();
        for r in &refs {
            match r.parse::<Feature>() {
                Ok(f) => features.push(f),
                Err(_) => {
                    // Warn but don't fail — per spec, unrecognized enum values fall back
                    log::warn!("unrecognized feature: {r}");
                }
            }
        }
        Ok(features)
    }

    fn parse_window_specs(&mut self) -> Result<Vec<WindowSpec>, ParseError> {
        let mut specs = Vec::new();
        loop {
            let dir_str = self.expect_ident()?;
            if let Ok(dir) = dir_str.parse::<Cardinal>() {
                let count = self.expect_number()? as u32;
                specs.push(WindowSpec {
                    direction: dir,
                    count,
                });
            }
            if !self.try_consume(TokenKind::Comma) {
                break;
            }
            // Check if next token is a cardinal (for multi-direction windows)
            if let TokenKind::Ident(ref s) = self.peek_kind() {
                if s.parse::<Cardinal>().is_err() {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(specs)
    }

    /// Consume tokens until we hit a comma, closing brace, or newline-level boundary.
    /// Returns the text representation.
    fn consume_value_text(&mut self) -> String {
        let mut parts = Vec::new();
        let mut depth = 0;

        loop {
            match self.peek_kind() {
                TokenKind::Comma if depth == 0 => break,
                TokenKind::RBrace if depth == 0 => break,
                TokenKind::Eof => break,
                // If we see a keyword at depth 0, it's the start of a new block
                TokenKind::Room | TokenKind::Floor | TokenKind::Roof | TokenKind::Facade
                | TokenKind::Landscape | TokenKind::Site | TokenKind::Style
                    if depth == 0 => break,
                TokenKind::LBrace | TokenKind::LBracket | TokenKind::LParen => {
                    depth += 1;
                    parts.push(self.tokens[self.pos].kind.to_string());
                    self.advance();
                }
                TokenKind::RBrace | TokenKind::RBracket | TokenKind::RParen => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    parts.push(self.tokens[self.pos].kind.to_string());
                    self.advance();
                }
                _ => {
                    parts.push(self.tokens[self.pos].kind.to_string());
                    self.advance();
                }
            }
        }

        parts.join(" ")
    }

    // --- Token manipulation ---

    fn peek_kind(&self) -> TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    fn check(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(&self.peek_kind()) == std::mem::discriminant(&kind)
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        let actual = self.peek_kind();
        if std::mem::discriminant(&actual) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(&format!("expected {expected}, got {actual}")))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.peek_kind() {
            TokenKind::Ident(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            // Some identifiers are also keywords in other contexts
            TokenKind::X => {
                self.advance();
                Ok("x".to_string())
            }
            other => Err(self.error(&format!("expected identifier, got {other}"))),
        }
    }

    fn expect_ident_or_string(&mut self) -> Result<String, ParseError> {
        match self.peek_kind() {
            TokenKind::Ident(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            TokenKind::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            other => Err(self.error(&format!("expected identifier or string, got {other}"))),
        }
    }

    fn expect_number(&mut self) -> Result<f64, ParseError> {
        match self.peek_kind() {
            TokenKind::Number(n) => {
                self.advance();
                Ok(n)
            }
            other => Err(self.error(&format!("expected number, got {other}"))),
        }
    }

    fn expect_unit(&mut self) -> Result<String, ParseError> {
        match self.peek_kind() {
            TokenKind::Unit(u) => {
                let u = u.clone();
                self.advance();
                Ok(u)
            }
            other => Err(self.error(&format!("expected unit, got {other}"))),
        }
    }

    fn try_unit(&mut self) -> Option<String> {
        if let TokenKind::Unit(u) = self.peek_kind() {
            let u = u.clone();
            self.advance();
            Some(u)
        } else {
            None
        }
    }

    fn try_consume(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn try_string_or_ident(&mut self) -> Option<String> {
        match self.peek_kind() {
            TokenKind::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Some(s)
            }
            TokenKind::Ident(s) if !matches!(self.tokens.get(self.pos + 1).map(|t| &t.kind), Some(TokenKind::Colon)) => {
                // Only consume if it's not a key:value pair start
                let next_is_brace = matches!(
                    self.tokens.get(self.pos + 1).map(|t| &t.kind),
                    Some(TokenKind::LBrace)
                );
                if next_is_brace {
                    None // It's a block name like "style tudor {", not the house name
                } else {
                    let s = s.clone();
                    self.advance();
                    Some(s)
                }
            }
            _ => None,
        }
    }

    fn error(&self, msg: &str) -> ParseError {
        let line = self
            .tokens
            .get(self.pos)
            .map(|t| t.span.line)
            .unwrap_or(0);
        ParseError {
            message: msg.to_string(),
            line,
        }
    }
}

/// Parse an OIL source string into a Program AST.
pub fn parse_oil(source: &str) -> Result<Program, ParseError> {
    let tokens = super::lexer::tokenize(source).map_err(|e| ParseError {
        message: e.message,
        line: e.line,
    })?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}
