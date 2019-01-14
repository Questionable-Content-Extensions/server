using AutoMapper;
using QCExtensions.Application.Interfaces.Mapping;
using QCExtensions.Application.Items.Models;
using QCExtensions.Domain.Entities;
using System;

namespace QCExtensions.Application.Comics.Models
{
	public class ComicDto : IHaveCustomMapping
	{
		public int Comic { get; set; }
		public bool HasData { get; set; }
		public DateTime? PublishDate { get; set; }
		public bool IsAccuratePublishDate { get; set; }
		public string Title { get; set; }
		public string Tagline { get; set; }
		public bool IsGuestComic { get; set; }
		public bool IsNonCanon { get; set; }
		public bool HasNoCast { get; set; }
		public bool HasNoLocation { get; set; }
		public bool HasNoStoryline { get; set; }
		public bool HasNoTitle { get; set; }
		public bool HasNoTagline { get; set; }
		public string News { get; set; }
		public int? Previous { get; set; }
		public int? Next { get; set; }

		public EditorDataDto EditorData { get; set; }
		public ItemWithNavigationDataDto[] Items { get; set; }
		public ItemWithNavigationDataDto[] AllItems { get; set; }

		public void CreateMappings(Profile configuration)
		{
			configuration.CreateMap<Comic, ComicDto>()
				.ForMember(dto => dto.Comic, m => m.MapFrom(c => c.Id))
				.ForMember(dto => dto.HasData, m => m.MapFrom((s, d) => true));
		}
	}
}
