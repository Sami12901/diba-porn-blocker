use std::collections::HashSet;

pub struct BlocklistManager {
    static_domains: HashSet<String>,
    custom_blacklist: HashSet<String>,
    custom_whitelist: HashSet<String>,
}

impl BlocklistManager {
    pub fn new() -> Self {
        let mut static_domains = HashSet::new();

        // Comprehensive base list of major adult networks, domains, cam sites, proxies & bypasses
        let base_domains = [
            // Top adult tube and video sites
            "pornhub.com", "pornhubpremium.com", "xvideos.com", "xnxx.com", "xhamster.com",
            "xhamsterlive.com", "youporn.com", "redtube.com", "tube8.com", "spankbang.com",
            "eporner.com", "tnaflix.com", "drtuber.com", "beeg.com", "nuvid.com",
            "txxx.com", "hclips.com", "sunporno.com", "hotmovs.com", "vporn.com",
            "empflix.com", "daftsex.com", "porn.com", "porn555.com", "hdporn92.com",
            "porn300.com", "fuq.com", "badjojo.com", "palimas.com", "thumbzilla.com",
            "pornhat.com", "porntube.com", "severeporn.com", "xtube.com", "porndoe.com",
            "netporn.com", "porn00.org", "anyporn.com", "czechav.com", "bravoteens.com",
            "brazzers.com", "naughtyamerica.com", "bangbros.com", "realitykings.com", "digitalplayground.com",
            "twistys.com", "mofos.com", "babes.com", "wicked.com", "evilangel.com",
            "kink.com", "fetlife.com", "playboy.com", "penthouse.com", "hustler.com",
            // Cam sites & live streaming
            "chaturbate.com", "myfreecams.com", "cam4.com", "camsoda.com", "livejasmin.com",
            "bongacams.com", "stripchat.com", "flirt4free.com", "streamate.com", "camversity.com",
            "imlive.com", "babestation.tv", "jerkmate.com", "camwhores.tv", "rec-tube.com",
            // Adult comic, hentai & anime
            "nhentai.net", "hentaihaven.xxx", "hanime.tv", "tsumino.com", "pururin.io",
            "hitomi.la", "e-hentai.org", "exhentai.org", "gelbooru.com", "danbooru.donmai.us",
            "rule34.xxx", "rule34hentai.net", "multporn.net", "simply-hentai.com", "luscious.net",
            // Adult dating & hookups
            "adultfriendfinder.com", "ashleymadison.com", "alt.com", "fling.com", "passion.com",
            "kasidie.com", "benaughty.com", "flirt.com", "hookup.com", "casualx.badpuppy.com",
            // Reddit adult subdomains / proxies & mirrors
            "redgifs.com", "erome.com", "erothots.com", "coomer.party", "coomer.su",
            "kemono.party", "kemono.su", "thothub.to", "fapello.com", "simpcity.su",
            "bunkr.is", "bunkr.to", "bunkr.si", "cyberdrop.me", "pixiv.xxx",
            // Regional explicit domains
            "javhd.com", "javlibrary.com", "javfinder.la", "missav.com", "jable.tv",
            "7mmtv.tv", "avgle.com", "tokyo-library.com", "caribbeancom.com", "1pondo.tv",
            "fanza.co.jp", "dmm.co.jp", "spankwire.com", "heavy-r.com", "motherless.com",
            // Additional popular explicit keywords domains
            "sex.com", "xxx.com", "boobs.com", "milf.com", "tits.com",
            "hardcoresex.com", "freeones.com", "xcafe.com", "cumlouder.com", "hqporner.com",
        ];

        for domain in base_domains {
            static_domains.insert(domain.to_lowercase());
        }

        Self {
            static_domains,
            custom_blacklist: HashSet::new(),
            custom_whitelist: HashSet::new(),
        }
    }

    /// Clean input and extract hostname
    pub fn extract_host(input: &str) -> String {
        let clean = input.trim().to_lowercase();
        let without_proto = if let Some(idx) = clean.find("://") {
            &clean[idx + 3..]
        } else {
            &clean
        };

        let without_path = without_proto.split(&['/', '?', '#', ':'][..]).next().unwrap_or(without_proto);
        let without_www = without_path.trim_start_matches("www.");
        without_www.trim().to_string()
    }

    /// Check if a domain (or any parent domain) is blocked
    pub fn is_domain_blocked(&self, raw_input: &str) -> bool {
        let host = Self::extract_host(raw_input);
        if host.is_empty() {
            return false;
        }

        // Whitelist has absolute priority
        if self.is_whitelisted(&host) {
            return false;
        }

        // Check exact match in custom blacklist
        if self.custom_blacklist.contains(&host) {
            return true;
        }

        // Check exact match in static blocklist
        if self.static_domains.contains(&host) {
            return true;
        }

        // Check parent domains (e.g., "m.video.pornhub.com" -> "video.pornhub.com" -> "pornhub.com")
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() > 2 {
            for i in 1..parts.len() - 1 {
                let parent = parts[i..].join(".");
                if self.is_whitelisted(&parent) {
                    return false;
                }
                if self.custom_blacklist.contains(&parent) || self.static_domains.contains(&parent) {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_whitelisted(&self, host: &str) -> bool {
        if self.custom_whitelist.contains(host) {
            return true;
        }
        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() > 2 {
            for i in 1..parts.len() - 1 {
                let parent = parts[i..].join(".");
                if self.custom_whitelist.contains(&parent) {
                    return true;
                }
            }
        }
        false
    }

    pub fn add_custom_blacklist(&mut self, domain: &str) {
        let host = Self::extract_host(domain);
        if !host.is_empty() {
            self.custom_blacklist.insert(host);
        }
    }

    pub fn remove_custom_blacklist(&mut self, domain: &str) {
        let host = Self::extract_host(domain);
        self.custom_blacklist.remove(&host);
    }

    pub fn add_custom_whitelist(&mut self, domain: &str) {
        let host = Self::extract_host(domain);
        if !host.is_empty() {
            self.custom_whitelist.insert(host);
        }
    }

    pub fn remove_custom_whitelist(&mut self, domain: &str) {
        let host = Self::extract_host(domain);
        self.custom_whitelist.remove(&host);
    }

    pub fn get_rules_count(&self) -> usize {
        self.static_domains.len() + self.custom_blacklist.len()
    }
}
