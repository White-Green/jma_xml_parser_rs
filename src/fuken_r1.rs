//! 府県天気予報（Ｒ１）

use std::convert::TryInto;

use chrono::{DateTime, Local};
use serde_derive::Deserialize;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Report {
    pub control: Control,
    pub head: Head,
    pub body: Vec<MeteorologicalInfoListItem>,
}

impl Report {
    fn from_tmp(tmp: ReportTmp) -> anyhow::Result<Self> {
        let ReportTmp {
            control,
            head,
            body: BodyTmp {
                meteorological_infos,
            },
        } = tmp;
        let mut body = Vec::new();
        for i in meteorological_infos
            .into_iter()
            .map(MeteorologicalInfoListItem::from_tmp)
        {
            body.extend(i?);
        }
        Ok(Report {
            control,
            head,
            body,
        })
    }
}

impl FromStr for Report {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tmp: ReportTmp = serde_xml_rs::from_str(s)?;
        Report::from_tmp(tmp)
    }
}

#[derive(Debug, Clone)]
pub enum MeteorologicalInfoListItem {
    /// 今日,明日,(明後日)の天気,風,波
    WeatherForecast(Vec<AreaForecast<WeatherForecast>>),
    /// 6時間毎ぐらいの降水確率
    ProbabilityOfPrecipitation(Vec<AreaForecast<Precipitation>>),
    /// 3時間毎の天気,風
    WeatherAndWindTimeSeries(Vec<AreaForecast<WeatherAndWindForecast>>),
    /// 向こう数日の気温
    TemperatureForecast(Vec<AreaForecast<TemperatureForecast>>),
    /// 3時間毎の気温
    TemperatureTimeSeries(Vec<AreaForecast<TemperatureTimeSeries>>),
    /// 独自予報
    Proprietary(Vec<MeteorologicalInfo>),
}

impl MeteorologicalInfoListItem {
    fn from_tmp(tmp: MeteorologicalInfosTmp) -> anyhow::Result<Vec<Self>> {
        let MeteorologicalInfosTmp {
            _type,
            time_series_info,
            meteorological_info,
        } = tmp;
        if _type == "独自予報" {
            return Ok(vec![MeteorologicalInfoListItem::Proprietary(
                meteorological_info,
            )]);
        }
        let mut result = Vec::new();
        for time_series_info in time_series_info {
            let TimeSeriesInfo {
                time_defines:
                    TimeDefines {
                        items: mut time_defines,
                    },
                items,
            } = time_series_info;
            time_defines.sort_by_key(|t| t.time_id);
            match _type.as_str() {
                "区域予報" => {
                    match items[0].kind[0].property[0]._type.as_str() {
                        "天気" | "波" | "風" => {
                            let mut forecasts = Vec::new();
                            for Item { kind, area } in items {
                                let properties = kind
                                    .into_iter()
                                    .flat_map(|kind| kind.property)
                                    .collect::<Vec<_>>();
                                match properties.len() {
                                    3 => {
                                        let mut properties: [_; 3] = properties.try_into().unwrap();
                                        properties.sort_by_key(|p| p._type.clone());
                                        if let [Property {
                                            _type: weather_type,
                                            detail_forecast:
                                                Some(DetailForecast {
                                                    mut weather_forecast_part,
                                                    ..
                                                }),
                                            weather_part: Some(WeatherPart { mut weather }),
                                            weather_code_part:
                                                Some(WeatherCodePart { mut weather_code }),
                                            ..
                                        }, Property {
                                            _type: wave_height_type,
                                            detail_forecast:
                                                Some(DetailForecast {
                                                    mut wave_height_forecast_part,
                                                    ..
                                                }),
                                            ..
                                        }, Property {
                                            _type: wind_type,
                                            detail_forecast:
                                                Some(DetailForecast {
                                                    mut wind_forecast_part,
                                                    ..
                                                }),
                                            ..
                                        }] = properties
                                        {
                                            assert_eq!(weather_type, "天気");
                                            assert_eq!(wave_height_type, "波");
                                            assert_eq!(wind_type, "風");
                                            weather_forecast_part.sort_by_key(|w| w.ref_id);
                                            weather.sort_by_key(|w| w.ref_id);
                                            weather_code.sort_by_key(|w| w.ref_id);
                                            wave_height_forecast_part.sort_by_key(|w| w.ref_id);
                                            wind_forecast_part.sort_by_key(|w| w.ref_id);
                                            let forecast = time_defines.iter().zip(weather_forecast_part).zip(weather).zip(weather_code).zip(wave_height_forecast_part).zip(wind_forecast_part)
                                                .map(|(((((time_define, weather_text), weather), weather_code), wave_height), wind)| WeatherForecast { time: time_define.clone(), weather_text, weather, weather_code, wind, wave_height: Some(wave_height) })
                                                .collect();
                                            forecasts.push(AreaForecast { area, forecast });
                                        } else {
                                            return Err(anyhow::Error::msg(
                                                "Property has no some property",
                                            ));
                                        }
                                    }
                                    2 => {
                                        let mut properties: [_; 2] =
                                            properties.try_into().map_err(|_| {
                                                anyhow::Error::msg("invalid count of property")
                                            })?;
                                        properties.sort_by_key(|p| p._type.clone());
                                        if let [Property {
                                            _type: weather_type,
                                            detail_forecast:
                                                Some(DetailForecast {
                                                    mut weather_forecast_part,
                                                    ..
                                                }),
                                            weather_part: Some(WeatherPart { mut weather }),
                                            weather_code_part:
                                                Some(WeatherCodePart { mut weather_code }),
                                            ..
                                        }, Property {
                                            _type: wind_type,
                                            detail_forecast:
                                                Some(DetailForecast {
                                                    mut wind_forecast_part,
                                                    ..
                                                }),
                                            ..
                                        }] = properties
                                        {
                                            assert_eq!(weather_type, "天気");
                                            assert_eq!(wind_type, "風");
                                            weather_forecast_part.sort_by_key(|w| w.ref_id);
                                            weather.sort_by_key(|w| w.ref_id);
                                            weather_code.sort_by_key(|w| w.ref_id);
                                            wind_forecast_part.sort_by_key(|w| w.ref_id);
                                            let forecast = time_defines.iter().zip(weather_forecast_part).zip(weather).zip(weather_code).zip(wind_forecast_part)
                                                .map(|((((time_define, weather_text), weather), weather_code), wind)| WeatherForecast { time: time_define.clone(), weather_text, weather, weather_code, wind, wave_height: None })
                                                .collect();
                                            forecasts.push(AreaForecast { area, forecast });
                                        } else {
                                            return Err(anyhow::Error::msg(
                                                "Property has no some property",
                                            ));
                                        }
                                    }
                                    _ => {
                                        return Err(anyhow::Error::msg("invalid count of property"))
                                    }
                                }
                            }
                            result.push(MeteorologicalInfoListItem::WeatherForecast(forecasts));
                        }
                        "降水確率" => {
                            let mut forecasts = Vec::new();
                            for Item { kind, area } in items {
                                let [property]: [_; 1] = kind
                                    .into_iter()
                                    .flat_map(|kind| kind.property)
                                    .collect::<Vec<_>>()
                                    .try_into()
                                    .map_err(|_| anyhow::Error::msg("invalid count of property"))?;
                                if let Property {
                                    _type,
                                    probability_of_precipitation_part:
                                        Some(ProbabilityOfPrecipitationPart { mut values }),
                                    ..
                                } = property
                                {
                                    values.sort_by_key(|p| p.ref_id);
                                    let forecast = time_defines
                                        .iter()
                                        .zip(values)
                                        .map(|(time, probability)| Precipitation {
                                            time: time.clone(),
                                            probability,
                                        })
                                        .collect();
                                    forecasts.push(AreaForecast { area, forecast });
                                } else {
                                    return Err(anyhow::Error::msg(
                                        "Property has no some property",
                                    ));
                                }
                            }
                            result.push(MeteorologicalInfoListItem::ProbabilityOfPrecipitation(
                                forecasts,
                            ));
                        }
                        "３時間内卓越天気" | "３時間内代表風" => {
                            let mut forecasts = Vec::new();
                            for Item { kind, area } in items {
                                let mut properties: [_; 2] = kind
                                    .into_iter()
                                    .flat_map(|kind| kind.property)
                                    .collect::<Vec<_>>()
                                    .try_into()
                                    .map_err(|_| anyhow::Error::msg("invalid count of property"))?;
                                properties.sort_by_key(|p| p._type.clone());
                                if let [Property {
                                    _type: wind_type,
                                    wind_direction_part:
                                        Some(WindDirectionPart {
                                            values: mut wind_directions,
                                        }),
                                    wind_speed_part:
                                        Some(WindSpeedPart {
                                            values: mut wind_speeds,
                                        }),
                                    ..
                                }, Property {
                                    _type: weather_type,
                                    weather_part: Some(WeatherPart { mut weather }),
                                    ..
                                }] = properties
                                {
                                    assert_eq!(wind_type, "３時間内代表風");
                                    assert_eq!(weather_type, "３時間内卓越天気");
                                    wind_directions.sort_by_key(|w| w.ref_id);
                                    wind_speeds.sort_by_key(|w| w.ref_id);
                                    weather.sort_by_key(|w| w.ref_id);
                                    let forecast = time_defines
                                        .iter()
                                        .zip(wind_directions)
                                        .zip(wind_speeds)
                                        .zip(weather)
                                        .map(|(((time, wind_direction), wind_speed), weather)| {
                                            WeatherAndWindForecast {
                                                time: time.clone(),
                                                weather,
                                                wind_direction,
                                                wind_speed,
                                            }
                                        })
                                        .collect();
                                    forecasts.push(AreaForecast { area, forecast });
                                } else {
                                    return Err(anyhow::Error::msg(
                                        "Property has no some property",
                                    ));
                                }
                            }
                            result.push(MeteorologicalInfoListItem::WeatherAndWindTimeSeries(
                                forecasts,
                            ));
                        }
                        _ => {}
                    }
                }
                "地点予報" => match items[0].kind[0].property[0]._type.as_str() {
                    "３時間毎気温" => {
                        let mut forecasts = Vec::new();
                        for Item { kind, area } in items {
                            let [property]: [_; 1] = kind
                                .into_iter()
                                .flat_map(|kind| kind.property)
                                .collect::<Vec<_>>()
                                .try_into()
                                .map_err(|_| anyhow::Error::msg("invalid count of property"))?;
                            if let Property {
                                _type,
                                temperature_part: Some(TemperaturePart { mut values }),
                                ..
                            } = property
                            {
                                values.sort_by_key(|p| p.ref_id);
                                let forecast = time_defines
                                    .iter()
                                    .zip(values)
                                    .map(|(time, temperature)| TemperatureTimeSeries {
                                        time: time.clone(),
                                        temperature,
                                    })
                                    .collect();
                                forecasts.push(AreaForecast { area, forecast });
                            } else {
                                return Err(anyhow::Error::msg("Property has no some property"));
                            }
                        }
                        result.push(MeteorologicalInfoListItem::TemperatureTimeSeries(forecasts));
                    }
                    _ => {
                        let mut forecasts = Vec::new();
                        for Item { kind, area } in items {
                            if kind.iter().flat_map(|kind| &kind.property).any(
                                |Property {
                                     temperature_part, ..
                                 }| {
                                    temperature_part.is_none()
                                        || temperature_part.as_ref().unwrap().values.is_empty()
                                },
                            ) {
                                return Err(anyhow::Error::msg("Property has no some property"));
                            }
                            let forecast = time_defines
                                .iter()
                                .zip(kind.into_iter().flat_map(|kind| kind.property))
                                .map(
                                    |(
                                        time,
                                        Property {
                                            _type,
                                            temperature_part,
                                            ..
                                        },
                                    )| TemperatureForecast {
                                        time: time.clone(),
                                        _type,
                                        temperature: temperature_part.unwrap().values.remove(0),
                                    },
                                )
                                .collect();
                            forecasts.push(AreaForecast { area, forecast });
                        }
                        result.push(MeteorologicalInfoListItem::TemperatureForecast(forecasts));
                    }
                },
                s => {
                    return Err(anyhow::Error::msg(format!(
                        "unknown TimeSeriesInfo::_type {}",
                        s
                    )))
                }
            }
        }
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct AreaForecast<T> {
    pub area: Area,
    pub forecast: Vec<T>,
}

#[derive(Debug, Clone)]
pub struct WeatherForecast {
    pub time: TimeDefine,
    pub weather_text: WeatherForecastPart,
    pub weather: Weather,
    pub weather_code: WeatherCode,
    pub wind: WindForecastPart,
    pub wave_height: Option<WaveHeightForecastPart>,
}

#[derive(Debug, Clone)]
pub struct Precipitation {
    pub time: TimeDefine,
    pub probability: ProbabilityOfPrecipitation,
}

#[derive(Debug, Clone)]
pub struct WeatherAndWindForecast {
    pub time: TimeDefine,
    pub weather: Weather,
    pub wind_direction: WindDirection,
    pub wind_speed: WindSpeedLevel,
}

#[derive(Debug, Clone)]
pub struct TemperatureForecast {
    pub time: TimeDefine,
    pub _type: String,
    pub temperature: Temperature,
}

#[derive(Debug, Clone)]
pub struct TemperatureTimeSeries {
    pub time: TimeDefine,
    pub temperature: Temperature,
}

#[derive(Debug, Clone, Deserialize)]
struct ReportTmp {
    #[serde(alias = "Control")]
    control: Control,
    #[serde(alias = "Head")]
    head: Head,
    #[serde(alias = "Body")]
    body: BodyTmp,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Control {
    #[serde(alias = "Title")]
    pub title: String,
    #[serde(alias = "DateTime")]
    pub datetime: DateTime<Local>,
    #[serde(alias = "Status")]
    pub status: ControlStatus,
    #[serde(alias = "EditorialOffice")]
    pub editorial_office: String,
    #[serde(alias = "PublishingOffice")]
    pub publishing_office: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ControlStatus {
    #[serde(alias = "通常")]
    Normal,
    #[serde(alias = "訓練")]
    Training,
    #[serde(alias = "試験")]
    Test,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Head {
    #[serde(alias = "Title")]
    pub title: String,
    #[serde(alias = "ReportDateTime")]
    pub report_datetime: DateTime<Local>,
    #[serde(alias = "TargetDateTime")]
    pub target_datetime: DateTime<Local>,
    #[serde(alias = "TargetDuration")]
    pub target_duration: String,
    #[serde(alias = "InfoType")]
    pub info_type: InfoType,
    #[serde(alias = "InfoKind")]
    pub info_kind: String,
    #[serde(alias = "InfoKindVersion")]
    pub info_kind_version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum InfoType {
    #[serde(alias = "発表")]
    Announcement,
    #[serde(alias = "訂正")]
    Correction,
    #[serde(alias = "遅延")]
    Delay,
}

#[derive(Debug, Clone, Deserialize)]
struct BodyTmp {
    #[serde(alias = "MeteorologicalInfos")]
    meteorological_infos: Vec<MeteorologicalInfosTmp>,
}

#[derive(Debug, Clone, Deserialize)]
struct MeteorologicalInfosTmp {
    #[serde(alias = "type")]
    _type: String,
    #[serde(alias = "TimeSeriesInfo")]
    #[serde(default)]
    time_series_info: Vec<TimeSeriesInfo>,
    #[serde(alias = "MeteorologicalInfo")]
    #[serde(default)]
    meteorological_info: Vec<MeteorologicalInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MeteorologicalInfo {
    #[serde(alias = "DateTime")]
    pub date_time: DateTime<Local>,
    #[serde(alias = "Duration")]
    pub duration: String,
    #[serde(alias = "Item")]
    pub item: MeteorologicalInfoItem,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MeteorologicalInfoItem {
    #[serde(alias = "Kind")]
    pub kind: MeteorologicalInfoKind,
    #[serde(alias = "Area")]
    pub area: Area,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MeteorologicalInfoKind {
    #[serde(alias = "Property")]
    pub property: MeteorologicalInfoProperty,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MeteorologicalInfoProperty {
    #[serde(alias = "Type")]
    pub _type: String,
    #[serde(alias = "Text")]
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TimeSeriesInfo {
    #[serde(alias = "TimeDefines")]
    time_defines: TimeDefines,
    #[serde(alias = "Item")]
    items: Vec<Item>,
}

#[derive(Debug, Clone, Deserialize)]
struct TimeDefines {
    #[serde(alias = "TimeDefine")]
    items: Vec<TimeDefine>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeDefine {
    #[serde(alias = "timeId")]
    pub time_id: usize,
    #[serde(alias = "DateTime")]
    pub datetime: DateTime<Local>,
    #[serde(alias = "Duration")]
    pub duration: Option<String>,
    #[serde(alias = "Name")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Item {
    #[serde(alias = "Kind")]
    kind: Vec<Kind>,
    #[serde(alias = "Area")]
    #[serde(alias = "Station")]
    area: Area,
}

#[derive(Debug, Clone, Deserialize)]
struct Kind {
    #[serde(alias = "Property")]
    property: Vec<Property>,
}

#[derive(Debug, Clone, Deserialize)]
struct Property {
    #[serde(alias = "Type")]
    _type: String,
    #[serde(alias = "DetailForecast")]
    detail_forecast: Option<DetailForecast>,
    #[serde(alias = "WeatherPart")]
    weather_part: Option<WeatherPart>,
    #[serde(alias = "WeatherCodePart")]
    weather_code_part: Option<WeatherCodePart>,
    #[serde(alias = "ProbabilityOfPrecipitationPart")]
    probability_of_precipitation_part: Option<ProbabilityOfPrecipitationPart>,
    #[serde(alias = "TemperaturePart")]
    temperature_part: Option<TemperaturePart>,
    #[serde(alias = "WindDirectionPart")]
    wind_direction_part: Option<WindDirectionPart>,
    #[serde(alias = "WindSpeedPart")]
    wind_speed_part: Option<WindSpeedPart>,
}

#[derive(Debug, Clone, Deserialize)]
struct DetailForecast {
    #[serde(alias = "WeatherForecastPart")]
    #[serde(default)]
    weather_forecast_part: Vec<WeatherForecastPart>,
    #[serde(alias = "WindForecastPart")]
    #[serde(default)]
    wind_forecast_part: Vec<WindForecastPart>,
    #[serde(alias = "WaveHeightForecastPart")]
    #[serde(default)]
    wave_height_forecast_part: Vec<WaveHeightForecastPart>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeatherForecastPart {
    #[serde(alias = "refID")]
    pub ref_id: usize,
    #[serde(alias = "Sentence")]
    pub sentence: String,
    #[serde(alias = "Base")]
    pub base: WeatherForecastBase,
    #[serde(alias = "Temporary")]
    #[serde(default)]
    pub temporary: Vec<Temporary>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeatherForecastBase {
    #[serde(alias = "Weather")]
    pub weather: Weather,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Temporary {
    #[serde(alias = "TimeModifier")]
    pub time_modifier: String,
    #[serde(alias = "Weather")]
    pub weather: Weather,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Weather {
    #[serde(alias = "refID")]
    pub ref_id: Option<usize>,
    #[serde(alias = "type")]
    pub _type: String,
    #[serde(alias = "$value")]
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindForecastPart {
    #[serde(alias = "refID")]
    pub ref_id: usize,
    #[serde(alias = "Sentence")]
    pub sentence: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WaveHeightForecastPart {
    #[serde(alias = "refID")]
    pub ref_id: usize,
    #[serde(alias = "Sentence")]
    pub sentence: String,
}

#[derive(Debug, Clone, Deserialize)]
struct WeatherPart {
    #[serde(alias = "Weather")]
    weather: Vec<Weather>,
}

#[derive(Debug, Clone, Deserialize)]
struct WeatherCodePart {
    #[serde(alias = "WeatherCode")]
    weather_code: Vec<WeatherCode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WeatherCode {
    #[serde(alias = "refID")]
    pub ref_id: usize,
    #[serde(alias = "type")]
    pub _type: String,
    #[serde(alias = "$value")]
    pub value: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct ProbabilityOfPrecipitationPart {
    #[serde(alias = "ProbabilityOfPrecipitation")]
    values: Vec<ProbabilityOfPrecipitation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProbabilityOfPrecipitation {
    pub condition: String,
    pub description: String,
    #[serde(alias = "refID")]
    pub ref_id: usize,
    #[serde(alias = "type")]
    pub _type: String,
    pub unit: String,
    #[serde(alias = "$value")]
    pub value: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct TemperaturePart {
    #[serde(alias = "Temperature")]
    values: Vec<Temperature>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Temperature {
    pub description: String,
    #[serde(alias = "refID")]
    pub ref_id: usize,
    #[serde(alias = "type")]
    pub _type: String,
    pub unit: String,
    #[serde(alias = "$value")]
    pub value: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct WindDirectionPart {
    #[serde(alias = "WindDirection")]
    values: Vec<WindDirection>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindDirection {
    #[serde(alias = "refID")]
    pub ref_id: usize,
    #[serde(alias = "type")]
    pub _type: String,
    pub unit: String,
    #[serde(alias = "$value")]
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
struct WindSpeedPart {
    #[serde(alias = "WindSpeedLevel")]
    values: Vec<WindSpeedLevel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindSpeedLevel {
    pub description: String,
    pub range: String,
    #[serde(alias = "refID")]
    pub ref_id: usize,
    #[serde(alias = "type")]
    pub _type: String,
    #[serde(alias = "$value")]
    pub value: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Area {
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "Code")]
    pub code: usize,
}
