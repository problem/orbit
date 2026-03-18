use super::ast::*;
use super::lexer::{Token, TokenKind};
use super::types::*;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug, thiserror::Error)]
#[error("parse error at line {line}, col {col}: {message}")]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
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

                // Try to parse structured values based on key name
                let value = if key.ends_with("_material") || key == "material" {
                    StyleValue::Material(self.parse_material_spec()?)
                } else if key.ends_with("_pitch") || key == "pitch" {
                    match self.try_parse_pitch() {
                        Some(p) => StyleValue::Pitch(p),
                        None => StyleValue::Text(self.consume_value_text()),
                    }
                } else {
                    StyleValue::Text(self.consume_value_text())
                };

                overrides.push(StyleProperty { key, value });
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
                    room.flooring = Some(self.parse_material_spec()?);
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

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Eof) {
            let key = self.expect_ident()?;

            // Handle continuation tokens without colon (e.g., "over [bedroom_2, bedroom_3]")
            if !self.check(TokenKind::Colon) {
                if key == "over" {
                    let rooms = self.parse_ref_list()?;
                    // Attach to the most recently parsed property
                    if let Some(ref mut d) = roof.dormers {
                        d.over = rooms;
                    } else if let Some(ref mut cg) = roof.cross_gable {
                        cg.over = Some(rooms.into_iter().next().unwrap_or_default());
                    }
                } else {
                    self.consume_value_text();
                }
                self.try_consume(TokenKind::Comma);
                continue;
            }

            self.expect(TokenKind::Colon)?;
            match key.as_str() {
                "primary" => roof.primary = Some(self.parse_roof_primary()?),
                "cross_gable" => roof.cross_gable = Some(self.parse_cross_gable()?),
                "dormers" => roof.dormers = Some(self.parse_dormer_spec()?),
                "material" => roof.material = Some(self.parse_material_spec()?),
                "pitch" => roof.pitch = self.try_parse_pitch().or_else(|| {
                    self.consume_value_text();
                    None
                }),
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

    /// Parse `gable(ridge: east-west)` or `hip` or `shed(south)`.
    fn parse_roof_primary(&mut self) -> Result<RoofPrimary, ParseError> {
        let form_name = self.expect_ident()?;
        let form = form_name
            .parse::<RoofForm>()
            .map_err(|e| self.error(&e))?;

        let mut params = Vec::new();
        if self.try_consume(TokenKind::LParen) {
            while !self.check(TokenKind::RParen) && !self.check(TokenKind::Eof) {
                let k = self.expect_ident()?;
                if self.try_consume(TokenKind::Colon) {
                    let v = self.expect_ident()?;
                    params.push((k, v));
                } else {
                    // Positional parameter like `shed(south)`
                    params.push(("direction".to_string(), k));
                }
                self.try_consume(TokenKind::Comma);
            }
            self.expect(TokenKind::RParen)?;
        }

        Ok(RoofPrimary { form, params })
    }

    /// Parse `over entry, pitch: 10:12`.
    fn parse_cross_gable(&mut self) -> Result<CrossGableSpec, ParseError> {
        let mut spec = CrossGableSpec {
            over: None,
            pitch: None,
        };

        // Expect "over <room_name>"
        if let TokenKind::Ident(ref s) = self.peek_kind() {
            if s == "over" {
                self.advance();
                spec.over = Some(self.expect_ident()?);
            }
        }

        // Optional ", pitch: N:N"
        if self.try_consume(TokenKind::Comma) {
            if let TokenKind::Ident(ref s) = self.peek_kind() {
                if s == "pitch" {
                    self.advance();
                    self.expect(TokenKind::Colon)?;
                    spec.pitch = self.try_parse_pitch();
                }
            }
        }

        Ok(spec)
    }

    /// Parse `2, over [bedroom_2, bedroom_3]` or just `2`.
    fn parse_dormer_spec(&mut self) -> Result<DormerSpec, ParseError> {
        let count = self.expect_number()? as u32;
        let mut over = Vec::new();

        // The "over [...rooms]" may follow after a comma
        if self.try_consume(TokenKind::Comma) {
            if let TokenKind::Ident(ref s) = self.peek_kind() {
                if s == "over" {
                    self.advance();
                    over = self.parse_ref_list()?;
                }
            }
        }

        Ok(DormerSpec { count, over })
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

    // --- Structured Value Parsers ---

    /// Parse `stucco("cream")` or `timber("dark oak")` or bare `hardwood`.
    fn parse_material_spec(&mut self) -> Result<MaterialSpec, ParseError> {
        let material_type = self.expect_ident()?;
        let color = if self.try_consume(TokenKind::LParen) {
            let c = match self.peek_kind() {
                TokenKind::StringLit(s) => {
                    let s = s.clone();
                    self.advance();
                    Some(s)
                }
                _ => {
                    // Non-string param like `casement(mullioned)` — consume as text
                    let text = self.consume_value_text();
                    Some(text)
                }
            };
            self.expect(TokenKind::RParen)?;
            c
        } else {
            None
        };
        Ok(MaterialSpec {
            material_type,
            color,
        })
    }

    /// Try to parse a pitch ratio like `12:12` or `10:12`.
    /// Returns None if the next tokens don't form a valid pitch.
    fn try_parse_pitch(&mut self) -> Option<Pitch> {
        if let TokenKind::Number(rise) = self.peek_kind() {
            let saved = self.pos;
            self.advance();
            if self.try_consume(TokenKind::Colon) {
                if let TokenKind::Number(run) = self.peek_kind() {
                    self.advance();
                    return Some(Pitch { rise, run });
                }
            }
            // Backtrack if not a valid pitch
            self.pos = saved;
        }
        None
    }

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
            // Approximate without unit (e.g., ~1.5 for aspect ratio)
            return Ok(ApproxValue::Approximate(num, Unit::Unitless));
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
                .unwrap_or(Unit::Unitless);
            return Ok(ApproxValue::Range(num, num2, unit));
        }

        if let Some(u) = unit_str {
            let unit = u.parse::<Unit>().map_err(|e| self.error(&e))?;
            Ok(ApproxValue::Exact(num, unit))
        } else {
            // Plain number (e.g., aspect ratio 1.5)
            Ok(ApproxValue::Exact(num, Unit::Unitless))
        }
    }

    fn parse_ref_list(&mut self) -> Result<Vec<String>, ParseError> {
        if self.try_consume(TokenKind::LBracket) {
            let mut refs = Vec::new();
            while !self.check(TokenKind::RBracket) && !self.check(TokenKind::Eof) {
                refs.push(self.expect_ident()?);
                self.try_consume(TokenKind::Comma);
            }
            self.expect(TokenKind::RBracket)?;
            Ok(refs)
        } else {
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

    /// Consume tokens until we hit a comma, closing brace, or block-level boundary.
    fn consume_value_text(&mut self) -> String {
        let mut parts = Vec::new();
        let mut depth = 0;

        loop {
            match self.peek_kind() {
                TokenKind::Comma if depth == 0 => break,
                TokenKind::RBrace if depth == 0 => break,
                TokenKind::Eof => break,
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
                let next_is_brace = matches!(
                    self.tokens.get(self.pos + 1).map(|t| &t.kind),
                    Some(TokenKind::LBrace)
                );
                if next_is_brace {
                    None
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
        let (line, col) = self
            .tokens
            .get(self.pos)
            .map(|t| (t.span.line, t.span.col))
            .unwrap_or((0, 0));
        ParseError {
            message: msg.to_string(),
            line,
            col,
        }
    }
}

/// Parse an OIL source string into a Program AST.
pub fn parse_oil(source: &str) -> Result<Program, ParseError> {
    let tokens = super::lexer::tokenize(source).map_err(|e| ParseError {
        message: e.message,
        line: e.line,
        col: e.col,
    })?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}
