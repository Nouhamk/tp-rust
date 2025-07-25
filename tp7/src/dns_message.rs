#[derive(Debug, Clone)]
pub struct DnsHeader {
    pub id: u16,
    pub flags: u16,
    pub question_count: u16,
    pub answer_count: u16,
    pub authority_count: u16,
    pub additional_count: u16,
}

#[derive(Debug, Clone)]
pub struct DnsQuestion {
    pub name: String,
    pub qtype: u16,  // 1 = A record
    pub qclass: u16, // 1 = IN (Internet)
}

#[derive(Debug, Clone)]
pub struct DnsAnswer {
    pub name: String,
    pub rtype: u16,
    pub rclass: u16,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct DnsMessage {
    pub header: DnsHeader,
    pub questions: Vec<DnsQuestion>,
    pub answers: Vec<DnsAnswer>,
}

impl DnsHeader {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            flags: 0x0100, // Standard query
            question_count: 0,
            answer_count: 0,
            authority_count: 0,
            additional_count: 0,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id.to_be_bytes());
        bytes.extend_from_slice(&self.flags.to_be_bytes());
        bytes.extend_from_slice(&self.question_count.to_be_bytes());
        bytes.extend_from_slice(&self.answer_count.to_be_bytes());
        bytes.extend_from_slice(&self.authority_count.to_be_bytes());
        bytes.extend_from_slice(&self.additional_count.to_be_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.len() < 12 {
            return Err("Header trop court".into());
        }
        
        Ok(Self {
            id: u16::from_be_bytes([bytes[0], bytes[1]]),
            flags: u16::from_be_bytes([bytes[2], bytes[3]]),
            question_count: u16::from_be_bytes([bytes[4], bytes[5]]),
            answer_count: u16::from_be_bytes([bytes[6], bytes[7]]),
            authority_count: u16::from_be_bytes([bytes[8], bytes[9]]),
            additional_count: u16::from_be_bytes([bytes[10], bytes[11]]),
        })
    }
}

impl DnsQuestion {
    pub fn new(name: String) -> Self {
        Self {
            name,
            qtype: 1,  // A record
            qclass: 1, // IN class
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Encoder le nom de domaine
        for part in self.name.split('.') {
            bytes.push(part.len() as u8);
            bytes.extend_from_slice(part.as_bytes());
        }
        bytes.push(0); // Null terminator
        
        bytes.extend_from_slice(&self.qtype.to_be_bytes());
        bytes.extend_from_slice(&self.qclass.to_be_bytes());
        
        bytes
    }

    pub fn from_bytes(bytes: &[u8], offset: &mut usize) -> Result<Self, Box<dyn std::error::Error>> {
        let name = Self::decode_name(bytes, offset)?;
        
        if *offset + 4 > bytes.len() {
            return Err("Question trop courte".into());
        }
        
        let qtype = u16::from_be_bytes([bytes[*offset], bytes[*offset + 1]]);
        let qclass = u16::from_be_bytes([bytes[*offset + 2], bytes[*offset + 3]]);
        *offset += 4;
        
        Ok(Self { name, qtype, qclass })
    }

    fn decode_name(bytes: &[u8], offset: &mut usize) -> Result<String, Box<dyn std::error::Error>> {
        let mut name_parts = Vec::new();
        
        while *offset < bytes.len() {
            let length = bytes[*offset] as usize;
            *offset += 1;
            
            if length == 0 {
                break;
            }
            
            if *offset + length > bytes.len() {
                return Err("Nom de domaine invalide".into());
            }
            
            let part = String::from_utf8(bytes[*offset..*offset + length].to_vec())?;
            name_parts.push(part);
            *offset += length;
        }
        
        Ok(name_parts.join("."))
    }
}

impl DnsAnswer {
    pub fn new(name: String, ip: [u8; 4]) -> Self {
        Self {
            name,
            rtype: 1,    // A record
            rclass: 1,   // IN class
            ttl: 300,    // 5 minutes
            rdlength: 4, // 4 bytes pour IPv4
            rdata: ip.to_vec(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Encoder le nom (simplifié)
        for part in self.name.split('.') {
            bytes.push(part.len() as u8);
            bytes.extend_from_slice(part.as_bytes());
        }
        bytes.push(0);
        
        bytes.extend_from_slice(&self.rtype.to_be_bytes());
        bytes.extend_from_slice(&self.rclass.to_be_bytes());
        bytes.extend_from_slice(&self.ttl.to_be_bytes());
        bytes.extend_from_slice(&self.rdlength.to_be_bytes());
        bytes.extend_from_slice(&self.rdata);
        
        bytes
    }
}

impl DnsMessage {
    pub fn new_query(id: u16, domain: String) -> Self {
        let mut header = DnsHeader::new(id);
        header.question_count = 1;
        
        let question = DnsQuestion::new(domain);
        
        Self {
            header,
            questions: vec![question],
            answers: vec![],
        }
    }

    pub fn new_response(query: &DnsMessage, answers: Vec<DnsAnswer>) -> Self {
        let mut header = query.header.clone();
        header.flags = 0x8180; // Response, authoritative
        header.answer_count = answers.len() as u16;
        
        Self {
            header,
            questions: query.questions.clone(),
            answers,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend_from_slice(&self.header.to_bytes());
        
        for question in &self.questions {
            bytes.extend_from_slice(&question.to_bytes());
        }
        
        for answer in &self.answers {
            bytes.extend_from_slice(&answer.to_bytes());
        }
        
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let header = DnsHeader::from_bytes(bytes)?;
        let mut offset = 12; // Taille de l'en-tête
        
        let mut questions = Vec::new();
        for _ in 0..header.question_count {
            let question = DnsQuestion::from_bytes(bytes, &mut offset)?;
            questions.push(question);
        }
        
        // Pour simplifier, on ne parse pas les réponses dans ce TP basique
        let answers = Vec::new();
        
        Ok(Self {
            header,
            questions,
            answers,
        })
    }
}